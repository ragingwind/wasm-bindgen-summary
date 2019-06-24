use futures::sync::oneshot;
use futures::Future;
use std::cell::{RefCell, UnsafeCell};
use std::mem;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};
use web_sys::{ErrorEvent, Event, Worker};

#[wasm_bindgen]
pub struct WorkerPool {
  state: Rc<PoolState>,
}

struct PoolState {
  worker: RefCell<Vec<Worker>>,
  callback: Closure<dyn FnMut(Event)>,
}

struct Work {
  func: Box<dyn FnOnce() + Send>,
}

#[wasm_bindgen]
impl WorkerPool {
  #[wasm_bindgen(constructor)]
  pub fn new(initial: usize) -> Result<WorkerPool, JsValue> {
    let pool = WorkerPool {
      state: Rc::new(PoolState {
        worker: RefCell::new(Vec::with_capacity(initial)),
        callback: Closure::wrap(Box::new(|event: Event| {
          console_log!("unhandled event: {}", event.type_());
          crate::logv(&event);
        }) as Box<dyn FnMut(Event)>),
      }),
    };

    for _ in 0..initial {
      let worker = pool.spawn()?;
      pool.state.push(worker);
    }

    Ok(pool)
  }

  fn spawn(&self) -> Result<Worker, JsValue> {
    console_log!("spawning new worker");

    let worker = Worker::new("./worker.js")?;
    let array = js_sys::Array::new();

    array.push(&wasm_bindgen::module());
    array.push(&wasm_bindgen::module());
    worker.post_message(&array)?;

    Ok(worker)
  }

  fn worker(&self, f: impl FnOnce() + Send + 'static) -> Result<Worker, JsValue> {
    let worker = self.worker()?;
    let work = Box::new(Work { func: Box::new(f) });
    let ptr = Box::into_raw(work);

    match worker.post_message(&JsValue::from(ptr as u32)) {
      Ok(()) => Ok(worker),
      Err(e) => {
        unsafe {
          drop(Box::from_raw(ptr));
        }
        Err(e)
      }
    }
  }

  fn reclaim_on_message(&self, worker: Worker, on_finish: impl FnOnce() + 'static) {
    let state = Rc::downgrade(&self.state);
    let worker2 = worker.clone();
    let reclaim_slot = Rc::new(RefCell::new(None));
    let slot2 = reclaim_slot.clone();
    let mut on_finish = Some(on_finish);
    let reclaim = Closure::Wrap(Box::new(move |event: Event| {
      if let Some(error) = event.dyn_ref::<ErrorEvent>() {
        console_log!("error in worker: {}", error.message());
        return;
      }

      if let Some(_msg) = event.dyn_ref::<MessageEvent>() {
        on_finish.take().unwrap()();
        if let Some(state) = state.upgrade() {
          state.push(worker.clone());
        }

        *slot2.borrow_mut() = None;
        return;
      }

      console_log!("unhandled event: {}", event.type_());
      crate::logv(&event);
    }) as   Box<dyn FnMut(Event)>);

    worker.set_onmessage(Some(reclaim.as_ref().unchecked_ref()));
    *reclaim_slot.borrow_mut() = Some(reclaim);
  }
}

impl WorkerPool {
  pub fn run(&self, f: impl FnOnce() + Send + 'static) -> Result<(), JsValue> {
    let worker = self.execute(f)?;
    self.reclaim_on_message(worker, || {});
    Ok(())
  }

  pub fn run_notify<T>(
        &self,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> Result<impl Future<Item = T, Error = JsValue> + 'static, JsValue>
    where
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let storage = Arc::new(AtomicValue::new(None));
        let storage2 = storage.clone();
        let worker = self.execute(move || {
            assert!(storage2.replace(Some(f())).is_ok());
        })?;
        self.reclaim_on_message(worker, move || match storage.replace(None) {
            Ok(Some(val)) => drop(tx.send(val)),
            _ => unreachable!(),
        });

        Ok(rx.map_err(|_| JsValue::undefined()))
    }
}

struct AtomicValue<T> {
    modifying: AtomicBool,
    slot: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for AtomicValue<T> {}
unsafe impl<T: Send> Sync for AtomicValue<T> {}

impl<T> AtomicValue<T> {
    fn new(val: T) -> AtomicValue<T> {
        AtomicValue {
            modifying: AtomicBool::new(false),
            slot: UnsafeCell::new(val),
        }
    }

    fn replace(&self, val: T) -> Result<T, T> {
        if self.modifying.swap(true, SeqCst) {
            return Err(val);
        }
        let ret = unsafe { mem::replace(&mut *self.slot.get(), val) };
        self.modifying.store(false, SeqCst);
        Ok(ret)
    }
}

impl PoolState {
    fn push(&self, worker: Worker) {
        worker.set_onmessage(Some(self.callback.as_ref().unchecked_ref()));
        worker.set_onerror(Some(self.callback.as_ref().unchecked_ref()));
        let mut workers = self.workers.borrow_mut();
        for prev in workers.iter() {
            let prev: &JsValue = prev;
            let worker: &JsValue = &worker;
            assert!(prev != worker);
        }
        workers.push(worker);
    }
}

/// Entry point invoked by `worker.js`, a bit of a hack but see the "TODO" above
/// about `worker.js` in general.
#[wasm_bindgen]
pub fn child_entry_point(ptr: u32) -> Result<(), JsValue> {
    let ptr = unsafe { Box::from_raw(ptr as *mut Work) };
    let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
    (ptr.func)();
    global.post_message(&JsValue::undefined())?;
    Ok(())
}

