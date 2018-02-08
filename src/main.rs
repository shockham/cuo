extern crate cargo;
extern crate git2;

use std::io;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};

use git2::{Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository};

use cargo::core::Workspace;
use cargo::ops;
use cargo::util::{CliResult, Config};
use cargo::util::important_paths::find_root_manifest_for_wd;

fn run() -> io::Result<()> {
    let cwd = env::current_dir()?;

    fs::read_dir(cwd)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|de| de.path())
        .filter(|path| {
            let mut toml_path = PathBuf::from(path);
            toml_path.push("Cargo.toml");

            let mut main_path = PathBuf::from(path);
            main_path.push("src/main.rs");

            toml_path.exists() && main_path.exists()
        })
        .for_each(|path| {
            if let Err(e) = check_repo(&path) {
                println!("cuo: {}\n==========", e);
            }
        });

    Ok(())
}

fn credentials_callback(
    url: &str,
    username: Option<&str>,
    allowed: git2::CredentialType,
    cfg: &git2::Config,
) -> Result<Cred, git2::Error> {
    if allowed.contains(git2::USERNAME) {
        return Err(git2::Error::from_str("Try usernames later"))
    }


    if allowed.contains(git2::SSH_KEY) {
        let mut cred_helper = git2::CredentialHelper::new(url);
        cred_helper.config(&cfg);

        let name = username
            .map(|s| s.to_string())
            .or_else(|| cred_helper.username.clone())
            .or_else(|| std::env::var("USER").ok())
            .or_else(|| std::env::var("USERNAME").ok())
            .or_else(|| Some("git".to_string()))
            .unwrap();

        let result = Cred::ssh_key_from_agent(&name);

        if result.is_ok() {
            return result
        }
    }

    if allowed.contains(git2::USER_PASS_PLAINTEXT) {
        if let Ok(token) = std::env::var("GH_TOKEN") {
            return Cred::userpass_plaintext(&token, "")
        } else if let Ok(cred_helper) = Cred::credential_helper(&cfg, url, username) {
            return Ok(cred_helper)
        }
    }

    if allowed.contains(git2::DEFAULT) {
        return Cred::default()
    }

    Err(git2::Error::from_str("cuo: no authentication available"))
}

fn check_repo(path: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("==========\ncuo: Checking: {:?}", repo.path());

    if !repo.statuses(None)?.is_empty() {
        return Err(git2::Error::from_str("Repo not clean"));
    }

    if !repo.is_path_ignored("Cargo.lock")? {
        println!("cuo: Updating rust bin project");
        let _ = cargo_update(path);

        if repo.statuses(None)?
            .iter()
            .any(|s| s.status() == git2::STATUS_WT_MODIFIED)
        {
            println!("cuo: Deps updated");
            let mut index = repo.index()?;
            index.add_all(Vec::<String>::new(), IndexAddOption::all(), None)?;
            let tree_id = index.write_tree()?;

            let sig = repo.signature()?;
            let tree = repo.find_tree(tree_id)?;

            let current_head = repo.head()?.peel_to_commit()?;

            // TODO better commit message describing what was updated
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Update deps",
                &tree,
                &[&current_head],
            )?;

            let cfg = repo.config()?;
            let mut rcbs = RemoteCallbacks::new();
            rcbs.credentials(|u,un,a| credentials_callback(u, un, a, &cfg));
            let mut push_ops = PushOptions::default();
            push_ops.remote_callbacks(rcbs);

            repo.find_remote("origin")?.push(&[], Some(&mut push_ops))?;
        }

        println!("cuo: Done!\n==========");
    }

    Ok(())
}

pub fn cargo_update(path: &Path) -> CliResult {
    let config = Config::default()?;
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
