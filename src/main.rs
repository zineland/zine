use anyhow::Result;

use zine::Parser;

fn main() -> Result<()> {
    let zine = Parser::new("demo");
    let site = zine.parse()?;
    println!("{:?}", site);
    Ok(())
}
