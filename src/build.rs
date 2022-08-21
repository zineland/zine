use std::{fs, path::Path, sync::mpsc, time::Duration};

use crate::{data, helpers::copy_dir, helpers::find_zine_folder, ZineEngine};
use anyhow::{Context, Result};
use notify::{watcher, RecursiveMode, Watcher};

pub async fn watch_build<P: AsRef<Path>>(source: P, dest: P, watch: bool) -> Result<()> {
    // Use `zine.toml` to find root path
    let source = find_zine_folder(&source.as_ref().to_path_buf())
        .with_context(|| "Failed to find zine folder".to_string())?;

    // Also make the dest folder joined in root path
    let dest = source.join(dest);

    data::load(&source);

    // Clone for moving
    let _source = source.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        // Save zine data only when the process gonna exist
        data::export(_source).unwrap();
        std::process::exit(0);
    });

    let build_result = _watch_build(source.as_ref(), dest.as_ref(), watch).await;
    if cfg!(debug_assertions) {
        // Explicitly panic build result in debug mode
        build_result.unwrap();
    } else if let Err(err) = build_result {
        println!("Error: {}", &err);
        std::process::exit(1);
    }
    Ok(())
}

async fn _watch_build(source: &Path, dest: &Path, watch: bool) -> Result<()> {
    let source = source.to_path_buf();
    let dest = dest.to_path_buf();

    // Spawn the build process as a blocking task, avoid starving other tasks.
    tokio::task::spawn_blocking(move || {
        build(&source, &dest)?;

        if watch {
            println!("Watching...");
            let (tx, rx) = mpsc::channel();
            let mut watcher = watcher(tx, Duration::from_secs(1))?;
            watcher.watch(&source, RecursiveMode::Recursive)?;

            // Watch zine's templates and static directory in debug mode to support reload.
            #[cfg(debug_assertions)]
            {
                watcher.watch("templates", RecursiveMode::Recursive)?;
                watcher.watch("static", RecursiveMode::Recursive)?;
            }

            loop {
                match rx.recv() {
                    Ok(_) => build(&source, &dest)?,
                    Err(err) => println!("watch error: {:?}", &err),
                }
            }
        }
        anyhow::Ok(())
    })
    .await?
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
