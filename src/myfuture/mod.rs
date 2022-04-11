pub mod myexecutor;
pub mod mytask;
pub mod mywaker;

use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Context,Waker,Poll},
    future::Future,
    time::Duration,
};

struct State {
    lapsed: bool,
    waker: Option<Waker>,
}
pub struct MyTimer {
    state: Arc<Mutex<State>>,
    str: String,
}
impl MyTimer {
    // since MyTimer impl Future
    // MyTimer::functions can be .awaited
    // causing MyTimer::poll() to be called
    pub fn new(lapse: u64) -> MyTimer {

        // initialise Timer struct
        let mt = MyTimer {
            state: Arc::new(Mutex::new(
                State {
                    lapsed: false,
                    waker: None,
                })),
            str: lapse.to_string(),
        };

        // extract pointer references
        let state = mt.state.clone();
        // spawn timer
        thread::spawn( move || {
            thread::sleep( Duration::new(lapse,0));
            // lock attr and change value
            let mut state = state.lock().unwrap();
            state.lapsed = true;
            // lock attr and callback using stored *fn()
            match state.waker.take() {
                None => println!("Finished without waker!!"),
                Some(w) => w.wake(),
            }
        });
        mt
    }
}
impl Future for MyTimer {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.lapsed {
            Poll::Ready(self.str.clone())
        } else {
            state.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}
