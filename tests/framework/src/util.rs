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

/// Write `data` to `path`. If `verify` is true, return an error in case `path`
/// changed.
pub fn write_bytes_and_verify_if_changed(
    path: &VfsPath,
    data: &[u8],
    verify: bool,
) -> anyhow::Result<()> {
    use crate::util::{read_bytes, write_bytes};
    use sha2::Digest as _;
    use sha2::Sha256;

    if verify {
        if !path.is_file()? {
            write_bytes(path, data)?;
            return Err(anyhow::anyhow!(
                "Output file '{}' does not exist or is not a file",
                path.as_str()
            ));
        }

        let mut existing_hash = Sha256::new();
        existing_hash.update(read_bytes(path)?);

        let mut new_hash = Sha256::new();
        new_hash.update(data);

        if existing_hash.finalize() != new_hash.finalize() {
            write_bytes(path, data)?;
            Err(anyhow::anyhow!(
                "Output file '{}' has changed during compilation",
                path.as_str()
            ))
        } else {
            Ok(())
        }
    } else {
        Ok(write_bytes(path, data)?)
    }
}
