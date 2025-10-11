use std::io::{Read, Write};
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
