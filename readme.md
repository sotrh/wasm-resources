# Managing resources with Web Assembly

This is a little demo on how to write a function to load a text file
and have it work on the web as well as on a desktop.

## The problem

While migrating my [WebGPU tutorial](https://sotrh.github.io/learn-wgpu)
to support WebAssembly, I ran into a snag in [the modle loading tutorial](https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models).
In that tutorial I use the [tobj crate](https://docs.rs/tobj) to load an obj
file that gets rendered with wgpu. This is all well and good in a native
context. All you need to do is give tobj a valid path, and then it's off
to the races.

On the web however, it's not that simple. For security reasons, a webpage
can't directly access your filesystem without permission. If your deploying
a game or other app to the web, your going to have to work around this issue.

With rust there are 2 options that I considered:

1. I could build the files I need into the program itself. This is what I used
   for the other tutorials. Using `include_string!` or `include_bytes!`
   I compile the contents of files such as images directly into the resulting
   binary.
2. I could host those files via a web server, and use http requests to fetch
   them when I want to load them.

While option #1 works for simple files such as text files and images, obj is
a little more complicated. An .obj file may be a text file, but it can 
optionally store path strings to material (.mtl) files. These files can also 
have path strings to different image files.

In order to build the obj into the binary, I would need to build all the 
material files and their corresponding images into the code as well. I'd need 
some way to get tobj to work with these built in files as well. At that point 
it would have made more sense to combile the vertex data of the obj directly 
into the binary through some custom macro. That would be a lot of code for a 
tutorial about loading models.

I decided to go with option #2 as I needed to use a web server to run the
resulting WebAssembly anyways.

## Dependencies

Here's my `Cargo.toml`:

```
[package]
name = "wasm-resources"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
gh_pages = []

[dependencies]
anyhow = "1"
cfg-if = "1"
lazy_static = "1"
log = "0.4"
pollster = "0.2"

[target.'cfg(target_arch = "wasm32")'.dependencies]
reqwest = "0.11"
wasm-bindgen = "0.2"
# Needed for async wasm_bindgen(start)
wasm-bindgen-futures = "0.4"
# These are only to make debugging easier
console_error_panic_hook = "0.1"
console_log = "0.2"
```

I opted to use [reqwest](https://docs.rs/reqwest) for this demo, as it's api is super simple to use. I could have probably used something like 
[web-sys](https://docs.rs/web-sys) and I may end up doing that in my tutorial
as I'm already using web-sys there, but I decided to keep things simple.

# The Rust code

```rust
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
                        "http://127.0.0.1:8000/res/".parse().unwrap()
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
```

The code is actually really simple. If you're ok with parsing the url/path in the `fetch_text_file` function directly, or you pass those in, then you don't
even need lazy_static.

I run the code natively as follows:

```rust
use std::env;

use wasm_resources::*;

fn main() -> anyhow::Result<()> {
    println!("{:#?}", env::current_dir()?);

    let text = pollster::block_on(fetch_text_file("wasm-tree.txt"))?;
    println!("The contents of \"wasm-tree.txt\":\n\n{}", text);

    Ok(())
}
```

The wasm-bindgen code is simple as well:

```rust
#[cfg(target_arch="wasm32")]
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn start() {
    console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Not the most useful demo, but it gets the point across
    let text = fetch_text_file("wasm-tree.txt").await.unwrap();
    log::info!("Contents of 'wasm-tree.txt':\n\n{}", text);
}
```

# The HTML code

Again, the code is really simple:

```html
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WASM Resource Management Demo</title>
</head>

<body>
    <h1>Managing resources with Web Assembly</h1>

    <p>
        Open up the console and you should see the contents of the 
        "wasm-tree.txt" text file.
    </p>

    <script type="module">
        import init from "./pkg/wasm_resources.js";
        init().then(() => {
            console.log("WASM Loaded");
        });
    </script>
</body>

</html>
```

I build the WASM using [wasm-pack](https://rustwasm.github.io/wasm-pack/), so
all I have to do is import the WASM code and just run it. I use the following
command to build the WebAssembly:

```sh
wasm-pack build --target web
```

## The server

For simple testing purposes I'm a fan of the simple python http server.

```sh
python3 -m http.server
```

This doesn't quite work as we need our server to send the appropriate CORS
headers. The following script I found on StackOverflow does the trick:

```python
#!/usr/bin/env python3

from http.server import HTTPServer, SimpleHTTPRequestHandler, test
import sys

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        SimpleHTTPRequestHandler.end_headers(self)

if __name__ == '__main__':
    test(CORSRequestHandler, HTTPServer, port=int(sys.argv[1]) if len(sys.argv) > 1 else 8000)
```

Obviously you can use what ever server you like. Building an loading the WASM
may differ, but the Rust code will stay the same.