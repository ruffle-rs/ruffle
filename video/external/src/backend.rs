use crate::decoder::openh264::OpenH264Codec;
use crate::decoder::VideoDecoder;
use bzip2::read::BzDecoder;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, PixelRegion};
use ruffle_video::backend::VideoBackend;
use ruffle_video::error::Error;
use ruffle_video::frame::{EncodedFrame, FrameDependency};
use ruffle_video::VideoStreamHandle;
use ruffle_video_software::backend::SoftwareVideoBackend;
use sha2::{Digest, Sha256};
use slotmap::SlotMap;
use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};
use swf::{VideoCodec, VideoDeblocking};

enum ProxyOrStream {
    /// These streams are passed through to the wrapped software
    /// backend, accessed using the stored ("inner") handle,
    /// which is completely internal to this backend.
    Proxied(VideoStreamHandle),

    /// These streams are handled by this backend directly.
    Owned(VideoStream),
}

struct OpenH264Data {
    local_filenames: Vec<&'static str>,
    download_filename: &'static str,
    download_sha256: &'static str,
}

/// A video backend that falls back to the software backend for most codecs,
/// except for H.264, for which it uses an external decoder.
pub struct ExternalVideoBackend {
    streams: SlotMap<VideoStreamHandle, ProxyOrStream>,
    openh264_codec: Option<OpenH264Codec>,
    software: SoftwareVideoBackend,
}

impl Default for ExternalVideoBackend {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ExternalVideoBackend {
    fn get_openh264_data() -> Result<OpenH264Data, Box<dyn std::error::Error>> {
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

    fn download_openh264(
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

    pub fn load_openh264(directory: &Path) -> Result<OpenH264Codec, Box<dyn std::error::Error>> {
        let openh264_data = Self::get_openh264_data()?;

        for filename in &openh264_data.local_filenames {
            match OpenH264Codec::new(filename) {
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
        let filename = Self::download_openh264(&openh264_data, directory)?;
        tracing::info!("Using OpenH264 at {:?}", filename);
        Ok(OpenH264Codec::new(&filename)?)
    }

    pub fn new(openh264_codec: Option<OpenH264Codec>) -> Self {
        Self {
            streams: SlotMap::with_key(),
            openh264_codec,
            software: SoftwareVideoBackend::new(),
        }
    }
}

// NOTE: The stream handles coming in through this API must not be
// conflated with the ones stored in `streams` as `Proxied`.
impl VideoBackend for ExternalVideoBackend {
    fn register_video_stream(
        &mut self,
        num_frames: u32,
        size: (u16, u16),
        codec: VideoCodec,
        filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        let proxy_or_stream = if codec == VideoCodec::H264 {
            if let Some(h264_codec) = self.openh264_codec.as_ref() {
                let decoder = Box::new(crate::decoder::openh264::H264Decoder::new(h264_codec));
                let stream = VideoStream::new(decoder);
                ProxyOrStream::Owned(stream)
            } else {
                return Err(Error::DecoderError("No OpenH264".into()));
            }
        } else {
            ProxyOrStream::Proxied(
                self.software
                    .register_video_stream(num_frames, size, codec, filter)?,
            )
        };

        Ok(self.streams.insert(proxy_or_stream))
    }

    fn configure_video_stream_decoder(
        &mut self,
        stream: VideoStreamHandle,
        configuration_data: &[u8],
    ) -> Result<(), Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or(Error::VideoStreamIsNotRegistered)?;

        match stream {
            ProxyOrStream::Proxied(handle) => self
                .software
                .configure_video_stream_decoder(*handle, configuration_data),
            ProxyOrStream::Owned(stream) => stream.decoder.configure_decoder(configuration_data),
        }
    }

    fn preload_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or(Error::VideoStreamIsNotRegistered)?;

        match stream {
            ProxyOrStream::Proxied(handle) => self
                .software
                .preload_video_stream_frame(*handle, encoded_frame),
            ProxyOrStream::Owned(stream) => stream.decoder.preload_frame(encoded_frame),
        }
    }

    fn decode_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
        renderer: &mut dyn RenderBackend,
    ) -> Result<BitmapInfo, Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or(Error::VideoStreamIsNotRegistered)?;

        match stream {
            ProxyOrStream::Proxied(handle) => {
                self.software
                    .decode_video_stream_frame(*handle, encoded_frame, renderer)
            }
            ProxyOrStream::Owned(stream) => {
                let frame = stream.decoder.decode_frame(encoded_frame)?;

                let w = frame.width();
                let h = frame.height();

                let handle = if let Some(bitmap) = stream.bitmap.clone() {
                    renderer.update_texture(&bitmap, frame, PixelRegion::for_whole_size(w, h))?;
                    bitmap
                } else {
                    renderer.register_bitmap(frame)?
                };
                stream.bitmap = Some(handle.clone());

                Ok(BitmapInfo {
                    handle,
                    width: w as u16,
                    height: h as u16,
                })
            }
        }
    }
}

/// A single preloaded video stream.
pub struct VideoStream {
    bitmap: Option<BitmapHandle>,
    decoder: Box<dyn VideoDecoder>,
}

impl VideoStream {
    fn new(decoder: Box<dyn VideoDecoder>) -> Self {
        Self {
            decoder,
            bitmap: None,
        }
    }
}
