use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use anyhow::Result;
use clap::StructOpt;
use notify::Watcher;
use tokio::{runtime::Runtime, task};
use tower_http::services::ServeDir;
use zine::{Builder, Parser};

#[derive(Debug, clap::Parser)]
#[clap(name = "zine")]
#[clap(about = "A simple and opinionated tool to build your own magazine.", long_about = None)]
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
        #[clap(short, long)]
        watch: bool,
    },
    /// Serve the zine site.
    #[clap(arg_required_else_help = true)]
    Serve {
        /// The source directory of zine site.
        source: String,
        /// The listen port.
        #[clap(short, default_value_t = 3000)]
        port: u16,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            source,
            dest,
            watch,
        } => {
            let dest = dest.unwrap_or_else(|| "build".into());
            watch_build(&source, &dest, watch)?;
        }
        Commands::Serve { source, port } => {
            let rt = Runtime::new()?;
            rt.block_on(async {
                let tmp_dir = env::temp_dir();
                let addr = SocketAddr::from(([127, 0, 0, 1], port));
                let service = ServeDir::new(&tmp_dir);
                task::spawn_blocking(move || {
                    watch_build(Path::new(&source), tmp_dir.as_path(), true).unwrap();
                });

                println!("listening on http://{}", addr.to_string());
                hyper::Server::bind(&addr)
                    .serve(tower::make::Shared::new(service))
                    .await
                    .expect("server error");
            });
        }
    }

    Ok(())
}

fn watch_build<P: AsRef<Path>>(source: P, dest: P, watch: bool) -> Result<()> {
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
    Ok(())
}

fn build<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    let site = Parser::new(source).parse()?;
    println!("{:?}", site);
    Builder::new(dest)?.build(site)?;
    fs::copy("target/zine.css", format!("{}/zine.css", dest.display()))
        .expect("File target/zine.css doesn't exists");
    copy_static_assets(source, dest)?;
    Ok(())
}

fn copy_static_assets<P: AsRef<Path>>(source: P, dest: P) -> Result<()> {
    let source = source.as_ref();
    let dest = PathBuf::from(dest.as_ref());
    for entry in walkdir::WalkDir::new(&format!("{}/static", source.display())) {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::create_dir_all(dest.join(path.strip_prefix(source)?))?;
        } else if path.is_file() {
            let to = dest.join(path.strip_prefix(source)?);
            fs::copy(path, to)?;
        }
    }
    Ok(())
}
