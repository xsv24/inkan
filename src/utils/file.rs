use std::{fs::File, io::Read, path::PathBuf};

use anyhow::Context;

pub fn get_file_contents(path: &PathBuf) -> anyhow::Result<String> {
    let file_name = path.as_os_str().to_str().unwrap_or_default();
    let mut buff = String::new();

    let mut reader = File::open(path).context(format!("Failed to open file at '{}'", file_name))?;

    reader
        .read_to_string(&mut buff)
        .context(format!("Failed to read file at '{}'", file_name))?;

    Ok(buff)
}
