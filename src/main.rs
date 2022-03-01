use std::{env, fs, sync::mpsc, time::Duration};

use anyhow::Result;

use notify::Watcher;
use zine::{Builder, Parser};

fn main() -> Result<()> {
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
    } else {
        build()?;
    }

    Ok(())
}

fn build() -> Result<()> {
    let site = Parser::new("demo").parse()?;
    println!("{:?}", site);
    Builder::new("dist")?.build(site)?;
    fs::copy("target/zine.css", "dist/zine.css").expect("File target/zine.css doesn't exists");
    Ok(())
}
