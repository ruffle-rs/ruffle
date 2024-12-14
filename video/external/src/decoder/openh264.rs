use core::slice;
use std::ffi::{c_int, c_uchar};
use std::fmt::Display;
use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;

use crate::decoder::openh264_sys::{self, videoFormatI420, ISVCDecoder, OpenH264};
use crate::decoder::VideoDecoder;

use ruffle_render::bitmap::BitmapFormat;
use ruffle_video::error::Error;
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};

use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct OpenH264Version(u32, u32, u32);

impl Display for OpenH264Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum OpenH264Error {
    #[error("Error while loading OpenH264: {0}")]
    LibraryLoadingError(#[from] ::libloading::Error),

    #[error("OpenH264 version mismatch, expected {0}, was {1}")]
    VersionMismatchError(OpenH264Version, OpenH264Version),
}

/// OpenH264 codec representation.
pub struct OpenH264Codec {
    openh264: Arc<OpenH264>,
}

impl OpenH264Codec {
    const VERSION: OpenH264Version = OpenH264Version(2, 4, 1);

    /// Returns the OpenH264 library data for the current platform.
    fn get_data() -> Result<OpenH264Data, Box<dyn std::error::Error>> {
        const OS: &str = std::env::consts::OS;
        const ARCH: &str = std::env::consts::ARCH;

        let local_filenames = match OS {
            "linux" => vec!["libopenh264.so.7", "libopenh264.so.2.4.1", "libopenh264.so"],
            // TODO: investigate other OSes
            _ => vec![],
        };

        // Source: https://github.com/cisco/openh264/releases/tag/v2.4.1
        let (download_filename, download_sha256) = match (OS, ARCH) {
            ("linux", "x86") => (
                "libopenh264-2.4.1-linux32.7.so",
                "b7cf0e407f99056d90cbf62787a34820a7595b2129b165319d50766e00a66704",
            ),
            ("linux", "x86_64") => (
                "libopenh264-2.4.1-linux64.7.so",
                "1392d21466bc638e68151b716d5b2086d54cd812afd43253f1adb5b6e0185f51",
            ),
            ("linux", "arm") => (
                "libopenh264-2.4.1-linux-arm.7.so",
                "fd1dfb27d30bb72e903c9d2b4c650104a4369d2e7ffe8a4a533e8db2e7e9b19e",
            ),
            ("linux", "aarch64") => (
                "libopenh264-2.4.1-linux-arm64.7.so",
                "e8ea7e42855ceb4a90e7bd0b3abeba0c58b5f97166e8b0a30eefd58e099557a4",
            ),
            ("macos", "x86_64") => (
                "libopenh264-2.4.1-mac-x64.dylib",
                "cc0ba518a63791c37571f3c851f0aa03a4fbda5410acc214ecd4f24f8d1c478e",
            ),
            ("macos", "aarch64") => (
                "libopenh264-2.4.1-mac-arm64.dylib",
                "213ff93831cfa3dd6d7ad0c3a3403a6ceedf4ac1341e1278b5b869d42fefb496",
            ),
            ("windows", "x86") => (
                "openh264-2.4.1-win32.dll",
                "83270149640469c994a62cc32a6d8c0413cd7b802b7f1f2f532159f5bdc1cedd",
            ),
            ("windows", "x86_64") => (
                "openh264-2.4.1-win64.dll",
                "081b0c081480d177cbfddfbc90b1613640e702f875897b30d8de195cde73dd34",
            ),
            (os, arch) => return Err(format!("Unsupported OS/arch: {}/{}", os, arch).into()),
        };

        Ok(OpenH264Data {
            local_filenames,
            download_filename,
            download_sha256,
        })
    }

    /// Downloads the OpenH264 library if it doesn't exist yet, and verifies its SHA256 hash.
    fn fetch_and_verify(
        openh264_data: &OpenH264Data,
        directory: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // See the license at: https://www.openh264.org/BINARY_LICENSE.txt
        const URL_BASE: &str = "http://ciscobinary.openh264.org/";
        const URL_SUFFIX: &str = ".bz2";

        let (filename, sha256sum) = (
            openh264_data.download_filename,
            openh264_data.download_sha256,
        );

        std::fs::create_dir_all(directory)?;
        let filepath = directory.join(filename);

        // If the binary doesn't exist in the expected location, download it.
        if !filepath.is_file() {
            let url = format!("{}{}{}", URL_BASE, filename, URL_SUFFIX);
            let response = reqwest::blocking::get(url)?;
            let mut bzip2_reader = BzDecoder::new(response);

            let mut tempfile = tempfile::NamedTempFile::with_prefix_in(filename, directory)?;
            copy(&mut bzip2_reader, &mut tempfile)?;
            // Let's assume that if this fails, it's because another process has already put it there
            // and loaded it, therefore it can't be overwritten (on Windows at least), but in the end,
            // all's fine - the hash will still be checked before attempting to load the library.
            let _ = tempfile.persist(&filepath);
        }

        // Regardless of whether the library was already there, or we just downloaded it, let's check the MD5 hash.
        let mut sha256 = Sha256::new();
        copy(&mut File::open(filepath.clone())?, &mut sha256)?;
        let sha256digest = sha256.finalize();
        let result: [u8; 32] = sha256digest.into();

        if result[..] != hex::decode(sha256sum)?[..] {
            let size = filepath.metadata().map(|f| f.len()).unwrap_or_default();
            return Err(format!(
                "SHA256 checksum mismatch for {filename}; expected {sha256sum}, found {sha256digest:x} (with a size of {size} bytes)",
            )
            .into());
        }

        Ok(filepath)
    }

    /// Loads an existing OpenH264 library from the given path.
    fn load_existing<P>(filename: P) -> Result<Self, OpenH264Error>
    where
        P: AsRef<::std::ffi::OsStr>,
    {
        let openh264 = unsafe { OpenH264::new(filename)? };

        let version = unsafe { openh264.WelsGetCodecVersion() };
        let version = OpenH264Version(version.uMajor, version.uMinor, version.uRevision);

        if Self::VERSION != version {
            return Err(OpenH264Error::VersionMismatchError(Self::VERSION, version));
        }

        Ok(Self {
            openh264: Arc::new(openh264),
        })
    }

    /// Loads the OpenH264 library - first trying one installed on the system (on supported platforms),
    /// then falling back to a local file in `directory`, downloading it into there if necessary.
    pub fn load(directory: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let openh264_data = Self::get_data()?;

        for filename in &openh264_data.local_filenames {
            match OpenH264Codec::load_existing(filename) {
                Ok(codec) => return Ok(codec),
                Err(err) => {
                    tracing::warn!(
                        "Failed to load system OpenH264 library {}: {}",
                        filename,
                        err
                    );
                }
            }
        }

        tracing::info!("Downloading OpenH264 library");
        let filename = Self::fetch_and_verify(&openh264_data, directory)?;
        tracing::info!("Using OpenH264 at {:?}", filename);
        Ok(OpenH264Codec::load_existing(&filename)?)
    }
}

/// H264 video decoder.
pub struct H264Decoder {
    /// How many bytes are used to store the length of the NALU (1, 2, 3, or 4).
    length_size: u8,

    openh264: Arc<OpenH264>,
    decoder: *mut ISVCDecoder,
}

struct OpenH264Data {
    local_filenames: Vec<&'static str>,
    download_filename: &'static str,
    download_sha256: &'static str,
}

impl H264Decoder {
    /// `extradata` should hold "AVCC (MP4) format" decoder configuration, including PPS and SPS.
    /// Make sure it has any start code emulation prevention "three bytes" removed.
    pub fn new(h264: &OpenH264Codec) -> Self {
        let openh264 = h264.openh264.clone();
        let mut decoder: *mut ISVCDecoder = ptr::null_mut();
        unsafe {
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
