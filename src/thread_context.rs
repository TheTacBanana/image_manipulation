use std::future::Future;

use futures::{channel::mpsc::{channel, Receiver, Sender}, executor::ThreadPool};

#[derive(Debug)]
pub struct ThreadContext {
    pub receiver: Receiver<Vec<u8>>,
    pub sender: Sender<Vec<u8>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub thread_pool: ThreadPool,
}

impl Default for ThreadContext {
    fn default() -> Self {
        let (sender, receiver) = channel(1);
        Self {
            receiver,
            sender,
            #[cfg(not(target_arch = "wasm32"))]
            thread_pool: ThreadPool::new().expect("Failed to create ThreadPool"),
        }
    }
}

impl ThreadContext {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute<F: Future<Output = ()> + Send + 'static>(&self, f: F) {
        self.thread_pool.spawn_ok(f);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn execute<F: Future<Output = ()> + 'static>(&self, f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
}