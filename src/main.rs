use std::{fs, path::PathBuf, sync::mpsc, time::Duration};

use anyhow::Result;
use clap::StructOpt;
use notify::Watcher;
use zine::{Builder, Parser};

#[derive(Debug, clap::Parser)]
#[clap(name = "zine")]
#[clap(about = "An simple and opinionated tool to build your own magazine.", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Build zine site.
    #[clap(arg_required_else_help = true)]
    Build {
        /// The source directory of zine site.
        source: String,
        /// The destination directory. Default dest dir is `build`.
        dest: Option<String>,
        /// Enable watching.
        #[clap(short = 'w', long = "watch")]
        watch: bool,
    },
    Serve,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            source,
            dest,
            watch,
        } => {
            let dest = dest.unwrap_or_else(|| "build".into());
            build(&source, &dest)?;

            if watch {
                println!("Watching...");
                let (tx, rx) = mpsc::channel();
                let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
                watcher.watch("templates", notify::RecursiveMode::Recursive)?;

                loop {
                    match rx.recv() {
                        Ok(_) => build(&source, &dest)?,
                        Err(err) => println!("watch error: {:?}", &err),
                    }
                }
            }
        }
        Commands::Serve => {}
    }

    Ok(())
}

fn build(source: &str, dest: &str) -> Result<()> {
    let site = Parser::new(source).parse()?;
    println!("{:?}", site);
    Builder::new(dest)?.build(site)?;
    fs::copy("target/zine.css", format!("{}/zine.css", dest))
        .expect("File target/zine.css doesn't exists");
    copy_static_assets(source, dest)?;
    Ok(())
}

fn copy_static_assets(source: &str, dest: &str) -> Result<()> {
    let dist = PathBuf::from(dest);
    for entry in walkdir::WalkDir::new(&format!("{}/static", source)) {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::create_dir_all(dist.join(path.strip_prefix(source)?))?;
        } else if path.is_file() {
            let to = dist.join(path.strip_prefix(source)?);
            fs::copy(path, to)?;
        }
    }
    Ok(())
}
