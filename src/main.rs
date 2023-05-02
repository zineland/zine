use anyhow::Result;
use genkit::Genkit;
use zine::{cmd, ZineGenerator, ZineMarkdownVisitor};

#[tokio::main]
async fn main() -> Result<()> {
    let generator = ZineGenerator {};
    let genkit = Genkit::new("zine", generator)
        .set_markdown_visitor(ZineMarkdownVisitor)
        .set_data_filename("zine-data.json")
        .set_banner(zine::ZINE_BANNER)
        .add_command(cmd::NewCmd);
    genkit.bootstrap().await?;
    Ok(())
}
