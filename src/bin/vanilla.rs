use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Context,Waker,Poll},
    future::Future,
    time::{Duration, Instant},
};

use async_test::myexecutor::MyExecutor;

struct State {
    lapsed: bool,
    waker: Option<Waker>,
}
struct MyTimer {
    state: Arc<Mutex<State>>,
}
impl MyTimer {
    // since MyTimer impl Future
    // MyTimer::functions can be .awaited
    // causing MyTimer::poll() to be called
    fn new(lapse: u64) -> MyTimer {

        // initialise Timer struct
        let mt = MyTimer {
            state: Arc::new(Mutex::new(
                State {
                    lapsed: false,
                    waker: None,
                }))
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
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.lapsed {
            Poll::Ready("done")
        } else {
            state.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}


async fn wait_timer(lapse: u64) -> &'static str {
    MyTimer::new(lapse).await
}

fn main() {
    let mut exec = MyExecutor::new();

    let now = Instant::now();

    for i in (1..=10).rev() {
        exec.spawn(async move {
            println!("F{}: {}", i, wait_timer(i).await);
        });
    }

    exec.run();
    println!("Total time: {:.2?}", now.elapsed() )
}