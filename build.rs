// Imported manually overwise we need a separate sub crate thats published.
#[path = "src/migrations.rs"]
mod migrations;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use migrations::{db_migrations, MigrationContext};
use rusqlite::Connection;

// Build script to copy over default templates & config into the binary directory.
fn main() {
    if let Some(dirs) = ProjectDirs::from("dev", "xsv24", "git-kit") {
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config_dir = dirs.config_dir();

        println!("Updating config file...");

        // Create config dir if not exists.
        fs::create_dir(config_dir).ok();

        copy_or_replace(
            &project_root.join(".git-kit.yml"),
            &config_dir.join(".git-kit.yml"),
        )
        .expect("Failed to copy or update to the latest config file for git-kit");

        let mut connection =
            Connection::open(config_dir.join("db")).expect("Failed to open sqlite connection");

        db_migrations(
            &mut connection,
            MigrationContext {
                config_path: config_dir.join(".git-kit.yml"),
                enable_side_effects: true,
                version: Some(2),
            },
        )
        .expect("Failed to apply migrations.");
        connection
            .close()
            .expect("Failed to close sqlite connection");
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
            println!(
                "copying from: {} {}, to: {} {}",
                &source_path.exists(),
                &source_path.display(),
                &target_path.exists(),
                &target_path.display()
            );
            fs::copy(source_path, target_path)?;
        }
    }

    Ok(())
}
