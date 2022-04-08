use std::{fs, path::Path, sync::mpsc, time::Duration};

use crate::{data, helpers::copy_dir, ZineEngine};
use anyhow::Result;
use notify::Watcher;

pub async fn watch_build<P: AsRef<Path>>(source: P, dest: P, watch: bool) -> Result<()> {
    if let Err(err) = _watch_build(source.as_ref(), dest.as_ref(), watch) {
        println!("Error: {}", &err);
        std::process::exit(1);
    }
    Ok(())
}

fn _watch_build(source: &Path, dest: &Path, watch: bool) -> Result<()> {
    data::load(&source);

    build(source, dest)?;

    let source_path_buf = source.to_path_buf();
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
        watcher.watch(&source, notify::RecursiveMode::Recursive)?;

        // Watch zine's templates and static directory in debug mode to support reload.
        #[cfg(debug_assertions)]
        {
            watcher.watch("templates", notify::RecursiveMode::Recursive)?;
            watcher.watch("static", notify::RecursiveMode::Recursive)?;
        }

        loop {
            match rx.recv() {
                Ok(_) => build(source, dest)?,
                Err(err) => println!("watch error: {:?}", &err),
            }
        }
    }
    Ok(())
}

fn build(source: &Path, dest: &Path) -> Result<()> {
    let instant = std::time::Instant::now();
    ZineEngine::new(source, dest)?.build()?;

    let static_dir = source.join("static");
    if static_dir.exists() {
        copy_dir(&static_dir, dest)?;
    }

    // Copy builtin static files into dest static dir.
    let dest_static_dir = dest.join("static");
    fs::create_dir_all(&dest_static_dir)?;

    #[cfg(not(debug_assertions))]
    include_dir::include_dir!("static").extract(dest_static_dir)?;
    // Alwasy copy static directory in debug mode.
    #[cfg(debug_assertions)]
    copy_dir(Path::new("./static"), dest)?;

    println!("Build cost: {}ms", instant.elapsed().as_millis());
    Ok(())
}
