# wasm-bindgen

> Rust library and CLI tool that faclitate high-level interactions between wasm modules and 
> Javascript.

## Hello, World

- `crate-type = ["cdylib"]` is largely used for wasm file final artifacts today
- Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) via curl
- wasm-pack supports a variety options of target to bundle wasm

## console.log

- `#[wasm_bindgen(start)]` will be running implicitly after loading wasm
- binding js apis by `#[wasm_bindgen(js_namespace = console, js_name = log)]`
- using `web_sys::console::log_1`

## Small wasm files

- `[wasm-bindgen]` generate small size bundle
- `wasm-opt` can make it even smaller
- `wasm2wat` can make it quite smaller
- using `lto`, Link Time Optimization option for release

```toml
[profile.release]
lto = true
```

## Converting WebAssembly to JS

- [Toolchain for WebAssembly](https://github.com/WebAssembly/binaryen), wasm2js, convert a wasm file to js
- `wasm2js pkg/wasm2js_bg.wasm -o pkg/wasm2js_bg.js`

## Importing non-browser JS

```rust
use wasm_bindgen::prelude::*;

// raw_module has been added, you can use relative path
#[wasm_bindgen(module = "/defined-in-js.js")]
extern "C" {
    #[wasm_bindgen(constructor)]
    fn new() -> MyClass;

    #[wasm_bindgen(method, getter)]
    fn number(this: &MyClass) -> u32;
    #[wasm_bindgen(method, setter)]
    fn set_number(this: &MyClass, number: u32) -> MyClass;
    #[wasm_bindgen(method)]
    fn render(this: &MyClass) -> String;
}
```