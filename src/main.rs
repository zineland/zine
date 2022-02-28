use anyhow::Result;

use zine::{Builder, Parser};

fn main() -> Result<()> {
    let zine = Parser::new("demo");
    let site = zine.parse()?;
    println!("{:?}", site);
    let builder = Builder::new("dist")?;
    builder.build(site)?;
    Ok(())
}
