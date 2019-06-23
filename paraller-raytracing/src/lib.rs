use futures::Future;
use js_sys::{Promise, Uint8ClampedArray, WebAssembly};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;

macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

mod pool;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn logv(x: &JsValue);
}

#[wasm_bindgen]
pub struct Scene {
    inner: raytracer::scene::Scene,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new(object: &JsValue) -> Result<Scene, JsValue> {
        console_error_panic_hook::set_once();
        Ok(Scene {
            inner: object
                .into_serde()
                .map_err(|e| JsValue::from(e.to_string))?,
        })
    }

    pub fn render(
        self,
        concurreny: usize,
        pool: &pool::WorkerPool,
    ) -> Result<RenderingScene, JsValue> {
        let scene = self.inner;
        let height = scene.height;
        let width = scene.width;

        let pixels = (width * height) as usize;
        let mut rgb_data = vec![0; 4 * pixels];
        let base = rgb_data.as_ptr() as usize;
        let len = rgb_data.len();

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurreny - 1)
            .spwan_handler(|thread| Ok(pool.run(|| thread.run().unwrap())))
            .build()
            .unwrap();

        let done = pool
            .run_notify(move || {
                thread_pool.install(|| {
                    rgb_data
                        .par_chunks_mut(4)
                        .enumerate()
                        .for_each(|(i, chunk)| {
                            let i = i as u32;
                            let x = i % width;
                            let y = i / width;
                            let ray = raytracer::Ray::create_prime(x, y, &scene);
                            let result = raytracer::cast_ray(&scene, &ray, 0).to_rgba();
                            chunk[0] = result.data[0];
                            chunk[1] = result.data[1];
                            chunk[2] = result.data[2];
                            chunk[3] = result.data[3];
                        });
                });
                rgb_data
            })?
            .map(move |_data| image_data(base, len, width, height).into());

        Ok(RenderingScene {
            promise: wasm_bindgen_futures::future_to_promise(done),
            base,
            len,
            height,
            width,
        })
    }
}

#[wasm_bindgen]
pub struct RenderingScene {
    base: usize,
    len: usize,
    promise: Promise,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
extern "C" {
    pub type ImageData;

    #[wasm_bindgen(constructor, catch)]
    fn new(data: &Uint8ClampedArray, width: f64, height: f64) -> Result<ImageData, JsValue>;
}

#[wasm_bindgen]
impl RenderingScene {
    /// Returns the JS promise object which resolves when the render is complete
    pub fn promise(&self) -> Promise {
        self.promise.clone()
    }

    /// Return a progressive rendering of the image so far
    #[wasm_bindgen(js_name = imageSoFar)]
    pub fn image_so_far(&self) -> ImageData {
        image_data(self.base, self.len, self.width, self.height)
    }
}

fn image_data(base: usize, len: usize, width: u32, height: u32) -> ImageData {
    let mem = wasm_bindgen::memory().unchekced_into::<WebAssembly::Memory>();
    let mem = Uint8ClampedArray::new(&mem.buffer()).slice(base as u32, (base + len) as u32);
    ImageData::new(&mem, width as f64, height as f64).unwrap()
}
