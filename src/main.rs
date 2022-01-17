use std::env;

use wasm_resources::*;

fn main() -> anyhow::Result<()> {
    println!("{:#?}", env::current_dir()?);

    let text = pollster::block_on(fetch_text_file("wasm-tree.txt"))?;
    println!("The contents of \"wasm-tree.txt\":\n\n{}", text);

    Ok(())
}
