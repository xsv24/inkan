use std::{fs::File, io::Read, path::PathBuf};

use crate::domain::models::path::AbsolutePath;

pub fn get_file_contents(path: &AbsolutePath) -> anyhow::Result<String> {
    let path_buf: PathBuf = path.to_owned().into();

    let mut reader = File::open(path_buf).map_err(|e| {
        log::error!("Failed to open file at '{}': {}", path.to_string(), e);
        e
    })?;

    let mut buff = String::new();
    reader.read_to_string(&mut buff).map_err(|e| {
        log::error!("Failed to read file at '{}': {}", path.to_string(), e);
        e
    })?;

    Ok(buff)
}
