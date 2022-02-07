use std::io::{BufReader, Cursor};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(target_arch="wasm32")] {
        lazy_static::lazy_static! {
            static ref BASE_URL: reqwest::Url = {
                // TODO: is there a better way to do this?
                cfg_if::cfg_if!{
                    if #[cfg(feature="gh_pages")] {
                        "https://sotrh.github.io/wasm-resources/res/".parse().unwrap()
                    } else {
                        "http://127.0.0.1:3000/res/".parse().unwrap()
                    }
                }
            };
        }
    } else {
        lazy_static::lazy_static!{
            static ref BASE_PATH: std::path::PathBuf = {
                std::path::PathBuf::from(format!("{}/res", std::env::current_dir().unwrap().as_os_str().to_str().unwrap()))
            };
        }
    }
}

pub async fn fetch_text_file(res_name: &str) -> anyhow::Result<String> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            let url = BASE_URL.join(res_name)?;
            let res = reqwest::get(url).await?;
            let text = res.text().await?;
        } else {
            let path = BASE_PATH.join(res_name);
            let text = std::fs::read_to_string(path)?;
        }
    }
    Ok(text)
}

pub async fn fetch_binary_file(res_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            let url = BASE_URL.join(res_name)?;
            let res = reqwest::get(url).await?;
            let data = res.bytes().await?.to_vec();
        } else {
            let path = BASE_PATH.join(res_name);
            let data = std::fs::read(path)?;
        }
    }
    Ok(data)
}

pub async fn fetch_obj(res_name: &str) -> anyhow::Result<(Vec<tobj::Model>, Vec<tobj::Material>)> {
    let obj_text = fetch_text_file(res_name).await?;
    let mut obj_reader = BufReader::new(Cursor::new(obj_text));
    let (models, materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let p = p.to_str().ok_or(tobj::LoadError::OpenFileFailed)?;
            let mtl_text = pollster::block_on(fetch_text_file(p)).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mtl_text)))
        }
    )?;

    Ok((models, materials?))
}

#[cfg(target_arch="wasm32")]
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn start() {
    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Not the most useful demo, but it gets the point across
    let text = fetch_text_file("wasm-tree.txt").await.unwrap();
    log::info!("Contents of 'wasm-tree.txt':\n\n{}", text);

    let (models, materials) = fetch_obj("cube.obj").await.unwrap();
    log::info!("Models for 'cube.obj':\n\n{:?}", models);
    log::info!("Materials for 'cube.obj':\n\n{:?}", materials);
}