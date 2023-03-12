use std::{path::Path, sync::mpsc, time::Duration};

use crate::{data, ZineEngine};
use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEventKind};
use tokio::sync::broadcast::Sender;

pub async fn watch_build<P: AsRef<Path>>(
    source: P,
    dest: P,
    watch: bool,
    sender: Option<Sender<()>>,
) -> Result<()> {
    // Use zine.toml to find root path
    let (source, zine) = crate::locate_root_zine_folder(std::fs::canonicalize(source)?)?
        .with_context(|| "Failed to find the root zine.toml file".to_string())?;

    // Also make the dest folder joined in root path?
    // let dest = source.join(dest);

    data::load(&source);

    let source_path = source.clone();

    let mut engine = ZineEngine::new(source, dest, zine)?;
    // Spawn the build process as a blocking task, avoid starving other tasks.
    let build_result = tokio::task::spawn_blocking(move || {
        engine.build(false)?;

        if let Some(sender) = sender.as_ref() {
            // Notify the first building finished.
            sender.send(())?;
        }

        if watch {
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                // Save zine data only when the process gonna exist
                data::export(source_path).unwrap();
                std::process::exit(0);
            });

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
                    Ok(result) => match result {
                        Ok(events) => {
                            // Prevent build too frequently, otherwise it will cause program stuck.
                            if events
                                .iter()
                                .any(|event| event.kind == DebouncedEventKind::Any)
                            {
                                match engine.build(true) {
                                    Ok(_) => {
                                        if let Some(sender) = sender.as_ref() {
                                            sender.send(())?;
                                        }
                                    }
                                    Err(err) => {
                                        println!("build error: {:?}", &err);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            println!("watch error: {:?}", &err);
                        }
                    },
                    Err(err) => println!("watch error: {:?}", &err),
                }
            }
        } else {
            data::export(source_path).unwrap();
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
