use std::{
    sync::{Arc, Mutex},
    pin::Pin,
    task::{Context, Poll},
    future::Future,
};
use futures::{
    task
};

use crate::mywaker::MyWaker;

pub struct MyTask {
    // has to be wrapped in Mutex so MyTask inherits the Send trait
    fut : Mutex<Pin<Box<dyn Future<Output=()> + Send>>>,
    pub(crate) waker: Arc<MyWaker>,
    pub(crate) lapsed: std::time::Instant
}

impl MyTask {
    pub fn new(fut: impl Future<Output=()> + Send + 'static) -> MyTask {
        MyTask {
            fut: Mutex::new(Box::pin(fut)),
            waker: Arc::new(MyWaker),
            lapsed: std::time::Instant::now()
        }
    }
    pub fn poll(&self) -> Poll<()> {

        let waker = task::waker(self.waker.clone());
        let mut cx = Context::from_waker( &waker);

        let mut f = self.fut.lock().unwrap();
        f.as_mut().poll(&mut cx)
    }
}
