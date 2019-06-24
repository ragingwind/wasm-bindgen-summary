use std::rc::Rc;
use wasm_bindgen::prelude::*;

pub mod controller;
pub mod element;
pub mod scheduler;
pub mod store;
pub mod template;
pub mod view;

use crate::controller::{Controller, ControllerMessage};
use crate::scheduler::Scheduler;
use crate::store::Store;
use crate::view::{View, ViewMessage};

pub enum Message {
  Controller(ControllerMessage),
  View(ViewMessage),
}

pub fn exit(message: &str) {
  let v = wasm_bindgen::JsValue::from_str(&message.to_string());
  web_sys::console::exception_1(&v);
  std::process::abort();
}

fn app(name: &str) {
  let sched = Rc::new(Scheduler::new());
  let store = match Store::new(name) {
    Some(s) => s,
    None => return,
  };

  let controller = Controller::new(store, Rc::downgrade(&sched));
  if let Some(mut view) = View::new(sched.clone()) {
    let sch: &Rc<Scheduler> = &sched;
    view.init();
    sch.set_view(view);
    sch.set_controller(controller);
    sched.add_message(Message::Controller(ControllerMessage::SetPage(
      "".to_string(),
    )))
  }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
  console_error_panic_hook::set_once();
  app("todos-wasmbindgen");

  Ok(())
}
