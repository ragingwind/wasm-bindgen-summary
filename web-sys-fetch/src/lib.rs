use futures::{future, Future};
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
  pub name: String,
  pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
  pub sha: String,
  pub commit: CommitDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitDetails {
  pub author: Signature,
  pub committer: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
  pub name: String,
  pub email: String,
}

#[wasm_bindgen]
pub fn run() -> Promise {
  let mut opts = RequestInit::new();
  opts.method("GET");
  opts.mode(RequestMode::Cors);
  
  let request = Request::new_with_str_and_init(
    "https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master",
    &opts,
  ).unwrap();

  request
    .headers()
    .set("Accept", "application/vnd.github.v3+json")
    .unwrap();

  let window = web_sys::window().unwrap();
  let request_promise = window.fetch_with_request(&request);

  let future = JsFuture::from(request_promise)
    .and_then(|resp_value| {
      assert!(resp_value.is_instance_of::<Response>());
      let resp: Response = resp_value.dyn_into().unwrap();
      resp.json()
    })
    .and_then(|json_value: Promise| {
      JsFuture::from(json_value)
    })
    .and_then(|json| {
      let branch_info: Branch = json.into_serde().unwrap();
      future::ok(JsValue::from_serde(&branch_info).unwrap())
    });

  future_to_promise(future)
}