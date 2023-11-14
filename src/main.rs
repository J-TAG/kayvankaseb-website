use clap::Parser;
use serde::Deserialize;
use std::fs;

/// Reckless CI command line tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Generate git local hooks if enabled in reckless.toml file
    #[arg(short, long)]
    generate_local_hooks: bool,
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

fn main() {
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
                println!("No `pre_commit` is set in reckless.toml");
                return;
            }
            Some(pre_commits) => {
                let mut data = "#!/bin/sh\nexec 1>&2\n".to_string();
                // Add all pre-commit commands to git hook
                for pre_commit in pre_commits {
                    // Commands should be executed using cli tool to show output in terminal
                    data.push_str(&format!(
                        "reckless-cli --pre-commit \"{}\"\n",
                        pre_commit.command
                    ));
                }
                data.push_str("exit 0\n");
                fs::write("/tmp/foo", data).expect("Unable to write file");
            }
        }
    }
}
