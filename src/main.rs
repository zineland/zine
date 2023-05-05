use anyhow::Result;
use clap::Command;
use genkit::Genkit;
use zine::{cmd, ZineGenerator, ZineMarkdownVisitor};

#[tokio::main]
async fn main() -> Result<()> {
    let command = Command::new(clap::crate_name!())
        .about(clap::crate_description!())
        .version(clap::crate_version!());
    Genkit::with_command(command, ZineGenerator)
        .markdown_visitor(ZineMarkdownVisitor)
        .data_filename("zine-data.json")
        .banner(zine::ZINE_BANNER)
        .add_command(cmd::NewCmd)
        .run()
        .await?;
    Ok(())
}
