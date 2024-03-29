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

## Working with `char`

- `#[wasm_bindgen] macro` will convert the rust char type to a single code-point js string

```rust
#[wasm_bindgen]
#[derive(Debug)]
pub struct Counter {
    key: char,
    count: i32,
}

...

pub fn increment(&mut self) {
    log("Counter.increment");
    self.count += 1;
}
```

## js-sys: WebAssembly in WebAssembly

- `const WASM: &[u8] = include_bytes!("add.wasm");`

## web-sys: DOM hello world

- enable all the various APIs

```
[dependencies.web-sys]
version = "0.3.4"
features = [
  'Document',
  'Element',
  'HtmlElement',
  'Node',
  'Window',
]
```

- get elements

```rust
let window = web_sys::window().expect("no global 'window' exists");
let document = window.document().expect("should have a document on window");
let body = document.body().expect("document should have a body");
```

## web-sys: Closure

- array function

```rust
array.for_each(&mut |obj, idx, _arr| match idx { ... });
```

- timer

```rust
update_time(&current_time);

let a = Closure::wrap(Box::new(move || update_time(&current_time)) as Box<dyn Fn()>);
window
    .set_interval_with_callback_and_timeout_and_arguments_0(a.as_ref().unchecked_ref(), 1000)?;

fn update_time(current_time: &Element) {
    current_time.set_inner_html(&String::from(
        Date::new_0().to_locale_string("en-GB", &JsValue::undefined()),
    ));
}
```

- click

```rust
let mut clicks = 0;
let a = Closure::wrap(Box::new(move || {
    clicks += 1;
}) as Box<dyn FnMut()>);

document
  .get_element_by_id("green-square")
  .dyn_ref::<HtmlElement>()
  .set_onclick(Some(a.as_ref().unchecked_ref()));
a.forget();
```

## web-sys: performance.now

```rust
let start = perf_to_system(performance.timing().request_start());
let end = perf_to_system(performance.timing().response_end());
```

## web-sys: The fetch API

```rust
let request = Request::new_with_str_and_init(
      "https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master",
      &opts,
  )
  .unwrap();

request
    .headers()
    .set("Accept", "application/vnd.github.v3+json")
    .unwrap();

let window = web_sys::window().unwrap();
let request_promise = window.fetch_with_request(&request);

let future = JsFuture::from(request_promise)
    .and_then(|resp_value| {
        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();
        resp.json()
    })
    .and_then(|json_value: Promise| {
        // Convert this other `Promise` into a rust `Future`.
        JsFuture::from(json_value)
    })
    .and_then(|json| {
        // Use serde to parse the JSON into a struct.
        let branch_info: Branch = json.into_serde().unwrap();

        // Send the `Branch` struct back to JS as an `Object`.
        future::ok(JsValue::from_serde(&branch_info).unwrap())
    });

// Convert this Rust `Future` back into a JS `Promise`.
future_to_promise(future)
```

## web-sys: 2D Canvas

```rust
let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
```

## web-sys: WebAudio

```rust
let ctx = web_sys::AudioContext::new()?;

// Create our web audio objects.
let primary = ctx.create_oscillator()?;
let fm_osc = ctx.create_oscillator()?;
let gain = ctx.create_gain()?;
let fm_gain = ctx.create_gain()?;

// Some initial settings:
primary.set_type(OscillatorType::Sine);
primary.frequency().set_value(440.0); // A4 note
gain.gain().set_value(0.0); // starts muted
fm_gain.gain().set_value(0.0); // no initial frequency modulation
fm_osc.set_type(OscillatorType::Sine);
fm_osc.frequency().set_value(0.0);
```

## web-sys: WebGL

```rust
let canvas = document.get_element_by_id("canvas").unwrap();
let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

let context = canvas
    .get_context("webgl")?
    .unwrap()
    .dyn_into::<WebGlRenderingContext>()?;

let vert_shader = compile_shader(
    &context,
    WebGlRenderingContext::VERTEX_SHADER,
    r#"
    attribute vec4 position;
    void main() {
        gl_Position = position;
    }
"#,
)?;
let frag_shader = compile_shader(
    &context,
    WebGlRenderingContext::FRAGMENT_SHADER,
    r#"
    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#,
)?;
let program = link_program(&context, &vert_shader, &frag_shader)?;
context.use_program(Some(&program));
```

## web-sys: WebSockets

```rust
let ws = WebSocket::new("wss://echo.websocket.org")?;

let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
    let response = e
        .data()
        .as_string()
        .expect("Can't convert received data to a string");
    console_log!("message event, received data: {:?}", response);
}) as Box<dyn FnMut(MessageEvent)>);
```

## web-sys: rAF

```rust
window()
      .request_animation_frame(f.as_ref().unchecked_ref())
      .expect("should register 'ra' ok");
```

## web-sys: paint

```rust
let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
    context.begin_path();
    context.move_to(event.offset_x() as f64, event.offset_y() as f64);
    pressed.set(true);
}) as Box<dyn FnMut(_)>);
canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
closure.forget();
```

## web-sys: Parallel Raytracing [WIP]

```rust
use web_sys::{ErrorEvent, Event, Worker};
...
let worker = Worker::new("./worker.js")?;
let array = js_sys::Array::new();
array.push(&wasm_bindgen::module());
array.push(&wasm_bindgen::memory());
worker.post_message(&array)?;
```

## TODO MVC using wasm-bingen and web-sys

- deletate API set for elements

```rust
impl Element {
    pub fn create_element(tag: &str) -> Option<Element> {
        if let Some(el) = web_sys::window()?.document()?.create_element(tag).ok() {
            Some(el.into())
        } else {
            None
        }
    }

    pub fn qs(selector: &str) -> Option<Element> {
        let body: web_sys::Element = web_sys::window()?.document()?.body()?.into();
        let el = body.query_selector(selector).ok()?;
        Some(Element { el })
    }

    pub fn add_event_listener<T>(&mut self, event_name: &str, handler: T)
    where
        T: 'static + FnMut(web_sys::Event),
    {
        let cb = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        if let Some(el) = self.el.take() {
            let el_et: EventTarget = el.into();
            el_et
                .add_event_listener_with_callback(event_name, cb.as_ref().unchecked_ref())
                .unwrap();
            cb.forget();
            if let Ok(el) = el_et.dyn_into::<web_sys::Element>() {
                self.el = Some(el);
            }
        }
    }
}
```

- event binding and managing for route changing

```rust
let set_page = Closure::wrap(Box::new(move || {
    if let Some(location) = document.location() {
        if let Ok(hash) = location.hash() {
            if let Ok(sched) = &(sched.try_borrow_mut()) {
                sched.add_message(Message::Controller(ControllerMessage::SetPage(hash)));
            }
        }
    }
}) as Box<dyn FnMut()>);

let window_et: web_sys::EventTarget = window.into();
window_et
    .add_event_listener_with_callback("hashchange", set_page.as_ref().unchecked_ref())
    .unwrap();
```
