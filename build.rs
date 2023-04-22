use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use directories::ProjectDirs;

// Build script to copy over default templates & config into the binary directory.
fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-env-changed=BUILD_DISABLED");
    // We might want to disable the build as "cross" uses docker images without sudo
    // This is sometimes needed for different distros to create our templates in the file system.
    let build_disabled = env::var("BUILD_DISABLED")
        .map(|v| v == "true")
        .unwrap_or(false);

    if build_disabled {
        println!("build.rs disabled exiting");
        return Ok(());
    }

    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "inkan") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config_dir = dirs.config_dir();
        println!("Updating config file... {}", config_dir.display());

        // Create config dir if not exists.
        fs::create_dir_all(config_dir).ok();

        copy_or_replace(&project_root.join("templates"), &config_dir.to_path_buf())
            .context("Failed to copy or update to the latest config file for inkan")?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn copy_or_replace(source_path: &PathBuf, target_path: &PathBuf) -> anyhow::Result<()> {
    match fs::read_dir(source_path) {
        Ok(entry_iter) => {
            fs::create_dir_all(target_path)
                .with_context(|| format!("Failed to create dir {:?}", target_path.as_os_str()))?;

            for dir in entry_iter {
                let entry = dir?;
                copy_or_replace(&entry.path(), &target_path.join(entry.file_name()))?;
            }
        }
        Err(_) => {
            println!(
                "copying from: {} {}, to: {} {}",
                &source_path.exists(),
                &source_path.display(),
                &target_path.exists(),
                &target_path.display()
            );
            fs::copy(source_path, target_path).with_context(|| {
                format!(
                    "Failed to copy from: {:?}, to: {:?}",
                    source_path.as_os_str(),
                    target_path.as_os_str()
                )
            })?;
        }
    }

    Ok(())
}
