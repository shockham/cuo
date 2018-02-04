extern crate cargo;
extern crate git2;

use std::io;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};

use git2::Repository;

use cargo::core::{Shell, Workspace};
use cargo::ops;
use cargo::util::{CliResult, Config};
use cargo::util::important_paths::find_root_manifest_for_wd;

fn run() -> io::Result<()> {
    let cwd = env::current_dir()?;

    // TODO: Clean up nesting
    for entry in fs::read_dir(cwd)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let mut toml_path = PathBuf::from(path.clone());
            toml_path.push("Cargo.toml");

            if toml_path.exists() {
                let mut main_path = PathBuf::from(path.clone());
                main_path.push("src/main.rs");

                if main_path.exists() {
                    let _ = check_repo(&path);
                }
            }
        }
    }

    Ok(())
}

fn check_repo(path: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("Checking: {:?}", repo.path());

    if !repo.is_path_ignored("Cargo.lock")? {
        println!("Possible rust bin");
        let _ = cargo_update(path);
    }

    Ok(())
}

pub fn cargo_update(path: &Path) -> CliResult {
    let mut config = Config::new(Shell::new(), path.into(), env::home_dir().unwrap());
    config.configure(0, Some(true), &Some("auto".into()), true, true, &Vec::new())?;

    let root = find_root_manifest_for_wd(None, path)?;

    let update_opts = ops::UpdateOptions {
        aggressive: false,
        precise: None,
        to_update: &Vec::new(),
        config: &config,
    };

    let ws = Workspace::new(&root, &config)?;
    ops::update_lockfile(&ws, &update_opts)?;
    Ok(())
}

fn main() {
    let _ = run();
}
