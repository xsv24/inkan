use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

// Build script to copy over default templates into the binary directory.
fn main() {
    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "git-kit") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = &env::var("CARGO_MANIFEST_DIR").unwrap();

        copy_or_replace(
            &Path::new(project_root).join("templates/"),
            &dirs.config_dir().join("templates/"),
        )
        .unwrap();
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn copy_or_replace(source_path: &PathBuf, target_path: &PathBuf) -> io::Result<()> {
    match fs::read_dir(source_path) {
        Ok(entry_iter) => {
            fs::create_dir_all(target_path)?;
            for dir in entry_iter {
                let entry = dir?;
                copy_or_replace(&entry.path(), &target_path.join(entry.file_name()))?;
            }
        }
        Err(_) => {
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}
