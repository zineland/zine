use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use zine::build::watch_build;
use zine::new::{new_article, new_zine_issue, new_zine_project};
use zine::serve::run_serve;
use zine::{lint, Mode};

#[derive(Debug, Parser)]
#[command(name = "zine")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build Zine site.
    Build {
        /// The source directory of zine site.
        source: Option<String>,
        /// The destination directory. Default dest dir is `build`.
        dest: Option<String>,
        /// Enable watching.
        #[arg(short, long)]
        watch: bool,
    },
    /// Serve the Zine site.
    Serve {
        /// The source directory of zine site.
        source: Option<String>,
        /// The listen port.
        #[arg(short, default_value_t = 3000)]
        port: u16,
        /// Auto open magazine in browser.
        #[arg(short, long)]
        open: bool,
    },
    /// New a Zine project.
    New {
        /// The project name.
        name: Option<String>,
        /// New issue.
        #[arg(short, long)]
        issue: bool,
        /// New article.
        #[arg(short, long)]
        article: bool,
    },
    /// Lint Zine project.
    Lint {
        /// The source directory of zine site.
        source: Option<String>,
        /// Enable CI mode. If lint failed will reture a non-zero code.
        #[arg(long)]
        ci: bool,
    },
    /// Prints the app version.
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            source,
            dest,
            watch,
        } => {
            zine::set_current_mode(Mode::Build);
            let dest = dest.unwrap_or_else(|| "build".into());
            watch_build(&source.unwrap_or_else(|| ".".into()), &dest, watch, None).await?;
            println!("Build success! The build directory is `{}`.", dest);
        }
        Commands::Serve { source, port, open } => {
            zine::set_current_mode(Mode::Serve);
            run_serve(source.as_deref().unwrap_or("."), port, open).await?;
        }
        Commands::New {
            name,
            issue,
            article,
        } => {
            if issue && article {
                bail!("Can't create both issue and article at the same time.")
            }

            if issue {
                new_zine_issue()?;
            } else if article {
                new_article()?;
            } else {
                new_zine_project(name)?
            }
        }
        Commands::Lint { source, ci } => {
            let success = lint::lint_zine_project(source.unwrap_or_else(|| ".".into())).await?;
            if ci && !success {
                std::process::exit(1);
            }
        }
        Commands::Version => {
            let version =
                option_env!("CARGO_PKG_VERSION").unwrap_or("(Unknown Cargo package version)");
            let date = option_env!("LAST_COMMIT_DATE").unwrap_or("");
            let build_info = env!("BUILD_INFO");
            println!("{}", zine::ZINE_BANNER);
            println!("Zine version {} {}", version, date);
            println!("({})", build_info);
        }
    }
    Ok(())
}
