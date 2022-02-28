use std::{env, sync::mpsc, time::Duration};

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
    let zine = Parser::new("demo");
    let site = zine.parse()?;
    println!("{:?}", site);
    let builder = Builder::new("dist")?;
    builder.build(site)
}
