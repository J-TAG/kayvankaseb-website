use clap::Parser;
use serde::Deserialize;
use std::{fs, process};
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;

/// Reckless CI command line tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Generate git local hooks if enabled in reckless.toml file
    #[arg(short, long)]
    generate_local_hooks: bool,
    /// Runs a pre-commit command
    #[arg(short, long)]
    pre_commit: String,
}

#[derive(Deserialize)]
struct Config {
    pre_commit: Option<Vec<PreCommit>>,
}

#[derive(Deserialize)]
struct PreCommit {
    name: String,
    command: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let config: Config = toml::from_str(
        r#"
   [[pre_commit]]
   name = 'test'
   command = 'cargo test'

   [[pre_commit]]
   name = 'build'
   command = 'cargo build'
"#,
    )
    .unwrap();

    if args.generate_local_hooks {
        match config.pre_commit {
            None => {
                eprintln!("No `pre_commit` is set in reckless.toml");
                process::exit(1);
            }
            Some(pre_commits) => {
                let mut data = "#!/bin/sh\nexec 1>&2\n".to_string();
                // Add all pre-commit commands to git hook
                for pre_commit in pre_commits {
                    // Commands should be executed using cli tool to show output in terminal
                    data.push_str(&format!(
                        "reckless-cli --pre-commit \"{}\"\n",
                        pre_commit.name
                    ));
                }
                data.push_str("exit 0\n");
                fs::write(".git/hooks/pre-commit", data)?;
                fs::set_permissions(".git/hooks/pre-commit", Permissions::from_mode(0o755))?;
            }
        }
    }

    if !args.pre_commit.is_empty() {
        let mut stream = UnixStream::connect("/tmp/reckless.sock")?;
        stream.write_all(args.pre_commit.as_ref())?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        println!("{response}");
    }

    Ok(())
}

fn read_config() -> String {
    return fs::read_to_string("./reckless.toml").expect("Unable to read config file");
}