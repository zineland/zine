use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use crate::{data, entity::Zine, error::ZineError, ZineEngine};
use anyhow::{anyhow, Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::broadcast::Sender;
use walkdir::WalkDir;

pub async fn watch_build<P: AsRef<Path>>(
    source: P,
    dest: P,
    watch: bool,
    sender: Option<Sender<()>>,
) -> Result<()> {
    // Use zine.toml to find root path
    let (source, zine) = locate_root_zine_folder(std::fs::canonicalize(source)?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;

    // Also make the dest folder joined in root path?
    // let dest = source.join(dest);

    data::load(&source);

    let source_path = source.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        // Save zine data only when the process gonna exist
        data::export(source_path).unwrap();
        std::process::exit(0);
    });

    let mut engine = ZineEngine::new(source, dest, zine)?;
    // Spawn the build process as a blocking task, avoid starving other tasks.
    let build_result = tokio::task::spawn_blocking(move || {
        build(&mut engine, false)?;

        if watch {
            println!("Watching...");
            let (tx, rx) = mpsc::channel();
            let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx)?;
            let watcher = debouncer.watcher();
            watcher.watch(&engine.source, RecursiveMode::Recursive)?;

            // Watch zine's templates and static directory in debug mode to support reload.
            #[cfg(debug_assertions)]
            {
                watcher.watch(Path::new("templates"), RecursiveMode::Recursive)?;
                watcher.watch(Path::new("static"), RecursiveMode::Recursive)?;
            }

            loop {
                match rx.recv() {
                    Ok(_) => match build(&mut engine, true) {
                        Ok(_) => {
                            if let Some(sender) = sender.as_ref() {
                                sender.send(())?;
                            }
                        }
                        Err(err) => {
                            println!("build error: {:?}", &err);
                        }
                    },
                    Err(err) => println!("watch error: {:?}", &err),
                }
            }
        }
        anyhow::Ok(())
    })
    .await?;

    if cfg!(debug_assertions) {
        // Explicitly panic build result in debug mode
        build_result.unwrap();
    } else if let Err(err) = build_result {
        println!("Error: {}", &err);
        std::process::exit(1);
    }
    Ok(())
}

fn build(engine: &mut ZineEngine, reload: bool) -> Result<()> {
    let instant = std::time::Instant::now();
    engine.build(reload)?;
    println!("Build cost: {}ms", instant.elapsed().as_millis());
    Ok(())
}

/// Find the root zine file in current dir and try to parse it
fn parse_root_zine_file<P: AsRef<Path>>(path: P) -> Result<Option<Zine>> {
    // Find the name in current dir
    if WalkDir::new(&path).max_depth(1).into_iter().any(|entry| {
        let entry = entry.as_ref().unwrap();
        entry.file_name() == crate::ZINE_FILE
    }) {
        // Try to parse the root zine.toml as Zine instance
        return Ok(Some(Zine::parse_from_toml(path)?));
    }

    Ok(None)
}

// Locate folder contains the root `zine.toml`, and return path info and Zine instance.
fn locate_root_zine_folder(path: PathBuf) -> Result<Option<(PathBuf, Zine)>> {
    match parse_root_zine_file(&path) {
        Ok(Some(zine)) => return Ok(Some((path, zine))),
        Err(err) => match err.downcast::<ZineError>() {
            // Found a root zine.toml, but it has invalid format
            Ok(inner_err @ ZineError::InvalidRootTomlFile(_)) => return Err(anyhow!(inner_err)),
            // Found a zine.toml, but it isn't a root zine.toml
            Ok(ZineError::NotRootTomlFile) => {}
            // No zine.toml file found
            _ => {}
        },
        _ => {}
    }

    match path.parent() {
        Some(parent_path) => locate_root_zine_folder(parent_path.to_path_buf()),
        None => Ok(None),
    }
}
