use core::slice;
use std::ffi::{c_int, c_uchar};
use std::ptr;

use crate::decoder::openh264_sys::{self, videoFormatI420, ISVCDecoder, OpenH264};
use crate::decoder::VideoDecoder;

use ruffle_render::bitmap::BitmapFormat;
use ruffle_video::error::Error;
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};

/// H264 video decoder.
pub struct H264Decoder {
    /// How many bytes are used to store the length of the NALU (1, 2, 3, or 4).
    length_size: u8,

    openh264: OpenH264,
    decoder: *mut ISVCDecoder,
}

impl H264Decoder {
    /// `extradata` should hold "AVCC (MP4) format" decoder configuration, including PPS and SPS.
    /// Make sure it has any start code emulation prevention "three bytes" removed.
    pub fn new(openh264_lib_filename: &std::path::Path) -> Self {
        let mut decoder: *mut ISVCDecoder = ptr::null_mut();
        unsafe {
            let openh264 = OpenH264::new(openh264_lib_filename).unwrap();

            let version = openh264.WelsGetCodecVersion();

            assert_eq!(
                (version.uMajor, version.uMinor, version.uRevision),
                (2, 4, 1),
                "Unexpected OpenH264 version"
            );

            openh264.WelsCreateDecoder(&mut decoder);

            let decoder_vtbl = (*decoder).as_ref().unwrap();

            let mut dec_param: openh264_sys::SDecodingParam = std::mem::zeroed();
            dec_param.sVideoProperty.eVideoBsType = openh264_sys::VIDEO_BITSTREAM_AVC;

            (decoder_vtbl.Initialize.unwrap())(decoder, &dec_param);

            Self {
                length_size: 0,
                openh264,
                decoder,
            }
        }
    }
}

impl Drop for H264Decoder {
    fn drop(&mut self) {
        unsafe {
            let decoder_vtbl = (*self.decoder).as_ref().unwrap();

            (decoder_vtbl.Uninitialize.unwrap())(self.decoder);
            self.openh264.WelsDestroyDecoder(self.decoder);
        }
    }
}

impl VideoDecoder for H264Decoder {
    fn configure_decoder(&mut self, configuration_data: &[u8]) -> Result<(), Error> {
        unsafe {
            // TODO: Check whether the "start code emulation prevention" needs to be
            // undone here before looking into the data. (i.e. conversion from SODB
            // into RBSP, by replacing each 0x00000301 byte sequence with 0x000001)

            assert_eq!(configuration_data[0], 1, "Invalid configuration version");
            // extradata[0]: configuration version, always 1
            // extradata[1]: profile
            // extradata[2]: compatibility
            // extradata[3]: level
            // extradata[4]: 6 reserved bits | NALU length size - 1

            self.length_size = (configuration_data[4] & 0b0000_0011) + 1;

            let decoder_vtbl = (*self.decoder).as_ref().unwrap();

            //input: encoded bitstream start position; should include start code prefix
            let mut buffer: Vec<c_uchar> = Vec::new();

            // Converting from AVCC to Annex B (stream-like) format,
            // putting the PPS and SPS into a NALU.

            buffer.extend_from_slice(&[0, 0, 0, 1]);

            let sps_length = configuration_data[6] as usize * 256 + configuration_data[7] as usize;

            for i in 0..sps_length {
                buffer.push(configuration_data[8 + i]);
            }

            let num_pps = configuration_data[8 + sps_length] as usize;

            assert_eq!(num_pps, 1, "More than one PPS is not supported");

            buffer.extend_from_slice(&[0, 0, 0, 1]);

            let pps_length = configuration_data[8 + sps_length + 1] as usize * 256
                + configuration_data[8 + sps_length + 2] as usize;

            for i in 0..pps_length {
                buffer.push(configuration_data[8 + sps_length + 3 + i]);
            }

            //output: [0~2] for Y,U,V buffer for Decoding only
            let mut output = [ptr::null_mut() as *mut c_uchar; 3];
            //in-out: for Decoding only: declare and initialize the output buffer info
            let mut dest_buf_info: openh264_sys::SBufferInfo = std::mem::zeroed();

            let _ret = decoder_vtbl.DecodeFrameNoDelay.unwrap()(
                self.decoder,
                buffer.as_mut_ptr(),
                buffer.len() as c_int,
                output.as_mut_ptr(),
                &mut dest_buf_info as *mut openh264_sys::SBufferInfo,
            );
        }
        Ok(())
    }

    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        assert!(self.length_size > 0, "Decoder not configured");

        let nal_unit_type = encoded_frame.data[self.length_size as usize] & 0b0001_1111;

        // 3.62 instantaneous decoding refresh (IDR) picture:
        // After the decoding of an IDR picture all following coded pictures in decoding order can
        // be decoded without inter prediction from any picture decoded prior to the IDR picture.
        if nal_unit_type == openh264_sys::NAL_SLICE_IDR as u8 {
            Ok(FrameDependency::None)
        } else {
            Ok(FrameDependency::Past)
        }
    }

    fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error> {
        assert!(self.length_size > 0, "Decoder not configured");
        unsafe {
            let decoder_vtbl = (*self.decoder).as_ref().unwrap();

            // input: encoded bitstream start position; should include start code prefix
            // converting from AVCC (file-like) to Annex B (stream-like) format
            // Thankfully the start code emulation prevention is there in both.
            let mut buffer: Vec<c_uchar> = Vec::with_capacity(encoded_frame.data.len());

            let mut i = 0;
            while i < encoded_frame.data.len() {
                let mut length = 0;
                for j in 0..self.length_size {
                    length = (length << 8) | encoded_frame.data[i + j as usize] as usize;
                }
                i += self.length_size as usize;
                buffer.extend_from_slice(&[0, 0, 1]);
                buffer.extend_from_slice(&encoded_frame.data[i..i + length]);
                i += length;
            }

            // output: [0~2] for Y,U,V buffer
            let mut output = [ptr::null_mut() as *mut c_uchar; 3];
            let mut dest_buf_info: openh264_sys::SBufferInfo = std::mem::zeroed();

            let ret = decoder_vtbl.DecodeFrameNoDelay.unwrap()(
                self.decoder,
                buffer.as_mut_ptr(),
                buffer.len() as c_int,
                output.as_mut_ptr(),
                &mut dest_buf_info as *mut openh264_sys::SBufferInfo,
            );

            if ret != 0 {
                return Err(Error::DecoderError(
                    format!("Decoding failed with status code: {}", ret).into(),
                ));
            }
            if dest_buf_info.iBufferStatus != 1 {
                return Err(Error::DecoderError(
                    "No output frame produced by the decoder".into(),
                ));
            }
            let buffer_info = dest_buf_info.UsrData.sSystemBuffer;
            if buffer_info.iFormat != videoFormatI420 as c_int {
                return Err(Error::DecoderError(
                    format!("Unexpected output format: {}", buffer_info.iFormat).into(),
                ));
            }

            let mut yuv: Vec<u8> = Vec::with_capacity(
                buffer_info.iWidth as usize * buffer_info.iHeight as usize * 3 / 2,
            );

            // Copying Y
            for i in 0..buffer_info.iHeight {
                yuv.extend_from_slice(slice::from_raw_parts(
                    output[0].offset((i * buffer_info.iStride[0]) as isize),
                    buffer_info.iWidth as usize,
                ));
            }

            // Copying U
            for i in 0..buffer_info.iHeight / 2 {
                yuv.extend_from_slice(slice::from_raw_parts(
                    output[1].offset((i * buffer_info.iStride[1]) as isize),
                    buffer_info.iWidth as usize / 2,
                ));
            }

            // Copying V
            for i in 0..buffer_info.iHeight / 2 {
                yuv.extend_from_slice(slice::from_raw_parts(
                    output[2].offset((i * buffer_info.iStride[1]) as isize),
                    buffer_info.iWidth as usize / 2,
                ));
            }

            // TODO: Check whether frames are being squished/stretched, or cropped,
            // when encoded image size doesn't match declared video tag size.
            // NOTE: This will always use the BT.601 coefficients, which may or may
            // not be correct. So far I haven't seen anything to the contrary in FP.
            Ok(DecodedFrame::new(
                buffer_info.iWidth as u32,
                buffer_info.iHeight as u32,
                BitmapFormat::Yuv420p,
                yuv,
            ))
        }
    }
}
