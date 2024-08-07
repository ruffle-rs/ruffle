use crate::decoder::VideoDecoder;
use bzip2::read::BzDecoder;
use md5::{Digest, Md5};
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, PixelRegion};
use ruffle_video::backend::VideoBackend;
use ruffle_video::error::Error;
use ruffle_video::frame::{EncodedFrame, FrameDependency};
use ruffle_video::VideoStreamHandle;
use ruffle_video_software::backend::SoftwareVideoBackend;
use slotmap::SlotMap;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use swf::{VideoCodec, VideoDeblocking};

enum ProxyOrStream {
    /// These streams are passed through to the wrapped software
    /// backend, accessed using the stored ("inner") handle,
    /// which is completely internal to this backend.
    Proxied(VideoStreamHandle),

    /// These streams are handled by this backend directly.
    Owned(VideoStream),
}

/// A video backend that falls back to the software backend for most codecs,
/// except for H.264, for which it uses an external decoder.
pub struct ExternalVideoBackend {
    streams: SlotMap<VideoStreamHandle, ProxyOrStream>,
    openh264_lib_filepath: Option<PathBuf>,
    software: SoftwareVideoBackend,
}

impl Default for ExternalVideoBackend {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ExternalVideoBackend {
    fn get_openh264_data() -> Result<(&'static str, &'static str), Box<dyn std::error::Error>> {
        // Source: https://github.com/cisco/openh264/releases/tag/v2.4.1
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86") => Ok((
                "libopenh264-2.4.1-linux32.7.so",
                "dd0743066117d63e1b2abc56a86506e5",
            )),
            ("linux", "x86_64") => Ok((
                "libopenh264-2.4.1-linux64.7.so",
                "19c561386a9564f8510fcb7586b9d402",
            )),
            ("linux", "arm") => Ok((
                "libopenh264-2.4.1-linux-arm.7.so",
                "2274a1bbd13f32b7afe22092e44fa2b5",
            )),
            ("linux", "aarch64") => Ok((
                "libopenh264-2.4.1-linux-arm64.7.so",
                "2aa205f08077aa2d049032e0b56c5b84",
            )),
            ("macos", "x86_64") => Ok((
                "libopenh264-2.4.1-mac-x64.dylib",
                "9fefa1e0279a49b8a4e9cf6fc148bc0c",
            )),
            ("macos", "aarch64") => Ok((
                "libopenh264-2.4.1-mac-arm64.dylib",
                "41f59bb5696ffeadbfba3a8a95ec39b7",
            )),
            ("windows", "x86") => Ok((
                "openh264-2.4.1-win32.dll",
                "a9360e6dd1e24320c3d65a0c65bf14a4",
            )),
            ("windows", "x86_64") => Ok((
                "openh264-2.4.1-win64.dll",
                "c85406e6b73812ec3fb9da5f898c6a9e",
            )),
            (os, arch) => Err(format!("Unsupported OS/ARCH: {} {}", os, arch).into()),
        }
    }

    pub fn get_openh264() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // See the license at: https://www.openh264.org/BINARY_LICENSE.txt
        const URL_BASE: &str = "http://ciscobinary.openh264.org/";
        const URL_SUFFIX: &str = ".bz2";

        let (filename, md5sum) = Self::get_openh264_data()?;

        let current_exe = std::env::current_exe()?;
        let directory = current_exe
            .parent()
            .ok_or("Could not determine Ruffle location.")?;
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
            // all's fine - the MD5 hash will still be checked before attempting to load the library.
            let _ = tempfile.persist(&filepath);
        }

        // Regardless of whether the library was already there, or we just downloaded it, let's check the MD5 hash.
        let mut md5 = Md5::new();
        copy(&mut File::open(filepath.clone())?, &mut md5)?;
        let md5digest = md5.finalize();
        let result: [u8; 16] = md5digest.into();

        if result[..] != hex::decode(md5sum)?[..] {
            let size = filepath.metadata().map(|f| f.len()).unwrap_or_default();
            return Err(format!(
                "MD5 checksum mismatch for {filename}; expected {md5sum}, found {md5digest:x} (with a size of {size} bytes)",
            )
            .into());
        }

        Ok(filepath)
    }

    pub fn new(openh264_lib_filepath: Option<PathBuf>) -> Self {
        Self {
            streams: SlotMap::with_key(),
            openh264_lib_filepath,
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
            let openh264 = &self.openh264_lib_filepath;
            if let Some(openh264) = openh264 {
                tracing::info!("Using OpenH264 at {:?}", openh264);
                let decoder = Box::new(crate::decoder::openh264::H264Decoder::new(openh264));
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
