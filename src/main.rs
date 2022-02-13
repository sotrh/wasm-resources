use std::env;

use wasm_resources::*;

async fn run() -> anyhow::Result<()> {
    let text = fetch_text_file("wasm-tree.txt").await?;
    println!("The contents of \"wasm-tree.txt\":\n\n{}", text);

    // let glb_name = "2 materials.glb";
    // let glb = fetch_glb(glb_name).await?;
    // println!("The contents of \"{}\": {:?}", glb_name, glb);

    let obj_name = "cube.obj";
    let obj = fetch_obj(obj_name).await?;
    println!("The contents of \"{}\": {:?}", obj_name, obj);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("{:#?}", env::current_dir()?);
    pollster::block_on(run())
}
