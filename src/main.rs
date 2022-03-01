use std::{env, fs, path::PathBuf, sync::mpsc, time::Duration};

use anyhow::Result;

use notify::Watcher;
use zine::{Builder, Parser};

fn main() -> Result<()> {
    build()?;

    if matches!(env::args().nth(1).as_deref(), Some("-w" | "--watch")) {
        println!("Watching...");

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        watcher.watch("templates", notify::RecursiveMode::Recursive)?;

        loop {
            match rx.recv() {
                Ok(_) => build()?,
                Err(err) => println!("watch error: {:?}", &err),
            }
        }
    }

    Ok(())
}

fn build() -> Result<()> {
    let site = Parser::new("demo").parse()?;
    println!("{:?}", site);
    Builder::new("dist")?.build(site)?;
    fs::copy("target/zine.css", "dist/zine.css").expect("File target/zine.css doesn't exists");
    copy_static_assets("demo", "dist")?;
    Ok(())
}

fn copy_static_assets(source: &str, dist: &str) -> Result<()> {
    let dist = PathBuf::from(dist);
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
