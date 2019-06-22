use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn raf(f: &Closure<dyn FnMut()>) {
    window()
      .request_animation_frame(f.as_ref().unchecked_ref())
      .expect("should register 'ra' ok");
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn body() -> web_sys::HtmlElement {
    document().body().unwrap()
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
      if i > 300 {
        body().set_text_content(Some("All done!"));

        let _ = f.borrow_mut().take();
        return;
      }

      i += 1;
      let text = format!("rFA has been called {} times", i);
      body().set_text_content(Some(&text));

      raf(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    raf(g.borrow().as_ref().unwrap());

    Ok(())
}