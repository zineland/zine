use anyhow::Result;
use clap::{Arg, Command};
use genkit::Genkit;
use zine::{
    new::{new_article, new_zine_issue, new_zine_project},
    ZineGenerator,
};

fn command_new() -> Command {
    Command::new("new")
        .args([Arg::new("name").help("The name of the project")])
        .about("New a Zine project.")
}

#[tokio::main]
async fn main() -> Result<()> {
    let generator = ZineGenerator {};
    let genkit = Genkit::new("zine", generator)
        .set_data_filename("zine-data.json")
        .set_banner(zine::ZINE_BANNER);
    genkit.bootstrap().await?;
    Ok(())
}
