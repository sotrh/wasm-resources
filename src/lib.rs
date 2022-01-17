#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(target_arch="wasm32")] {
        lazy_static::lazy_static! {
            // TODO: make the url/port configurable
            static ref BASE_URL: reqwest::Url = {
                "http://127.0.0.1:8000/res/".parse().unwrap()
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

#[cfg(target_arch="wasm32")]
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn start() {
    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Not the most useful demo, but it gets the point across
    let text = fetch_text_file("wasm-tree.txt").await.unwrap();
    log::info!("Contents of 'wasm-tree.txt':\n\n{}", text);
}