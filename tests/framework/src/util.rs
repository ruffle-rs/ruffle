use image::{EncodableLayout, ImageBuffer, ImageFormat, Pixel, PixelWithColorType};
use std::io::{Cursor, Read, Write};
use std::ops::Deref;
use vfs::{VfsError, VfsPath};

pub fn read_bytes(path: &VfsPath) -> Result<Vec<u8>, VfsError> {
    let mut bytes = Vec::with_capacity(path.metadata()?.len as usize);
    path.open_file()?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn write_bytes(path: &VfsPath, data: &[u8]) -> Result<(), VfsError> {
    path.create_file()?.write_all(data)?;
    Ok(())
}

pub fn write_image<P, Container>(
    path: &VfsPath,
    image: &ImageBuffer<P, Container>,
    format: ImageFormat,
) -> anyhow::Result<()>
where
    P: Pixel + PixelWithColorType,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut buffer = vec![];
    image.write_to(&mut Cursor::new(&mut buffer), format)?;
    write_bytes(path, &buffer)?;
    Ok(())
}
