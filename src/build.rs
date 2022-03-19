use std::{path::Path, sync::mpsc, time::Duration};

use crate::{data, helpers::copy_dir, ZineEngine};
use anyhow::Result;
use notify::Watcher;

pub async fn watch_build<P: AsRef<Path>>(source: P, dest: P, watch: bool) -> Result<()> {
    data::load(&source);

    build(&source, &dest)?;

    let source_path_buf = source.as_ref().to_path_buf();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        // Save zine data only when the process gonna exist
        data::export(source_path_buf).unwrap();
        std::process::exit(0);
    });

    if watch {
        println!("Watching...");
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        // watcher.watch("templates", notify::RecursiveMode::Recursive)?;
        watcher.watch(&source, notify::RecursiveMode::Recursive)?;

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

    ZineEngine::new(source, dest)?.build()?;

    let static_dir = source.join("static");
    if static_dir.exists() {
        copy_dir(&static_dir, dest)?;
    }
    copy_dir(Path::new("./static"), dest)?;
    Ok(())
}
