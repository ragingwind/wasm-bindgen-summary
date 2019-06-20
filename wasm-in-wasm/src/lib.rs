use js_sys::{Function, Object, Reflect, Uint8Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

const WASM: &[u8] = include_bytes!("add.wasm");

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_log!("instantiating a new wasm module directly");
    
    let a = unsafe {
        let array = Uint8Array::view(WASM);
        WebAssembly::Module::new(array.as_ref())?
    };
    let b = WebAssembly::Instance::new(&a, &Object::new())?;
    let c = b.exports();

    let add = Reflect::get(c.as_ref(), &"add".into())?
        .dyn_into::<Function>()
        .expect("add export wasn't a function");

    let three = add.call2(&JsValue::undefined(), &1.into(), &2.into())?;
    console_log!("1 + 2 = {:?}", three);
    let mem = Reflect::get(c.as_ref(), &"memory".into())?
        .dyn_into::<WebAssembly::Memory>()
        .expect("memory export wasn't a `WebAssembly.Memoty`");
    console_log!("created module has {} pages of memoty", mem.grow(0));
    console_log!("giving the module 4 more pages of memoty");
    mem.grow(4);
    console_log!("created module has {} pages of memoty", mem.grow(0));

    Ok(())
}