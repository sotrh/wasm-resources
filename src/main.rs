use std::env;

use wasm_resources::*;

fn main() -> anyhow::Result<()> {
    println!("{:#?}", env::current_dir()?);

    let text = pollster::block_on(fetch_text_file("data.json"))?;
    println!("The contents of \"index.html\":\n\n{}", text);

    Ok(())
}
