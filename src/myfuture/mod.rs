pub mod myexecutor;
pub mod mytask;

use std::{
    thread::{self,JoinHandle},
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Context,Poll},
    future::Future,
    time::Duration,
};

type Data = (Duration, bool, Option<JoinHandle<()>>);
pub struct MyTimer(Arc<Mutex<Data>>);

impl MyTimer {
    pub fn start(timeout: Duration) -> impl Future<Output = Duration> {
        // println!("Timer::start()");
        MyTimer(Arc::new(Mutex::new((timeout, false, None))))
    }
}

impl Future for MyTimer {
    type Output = Duration;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.lock().unwrap();
        if state.1 {
            // println!("Timer::Lapsed");
            return Poll::Ready( state.0 );
        } else {
            // have we spawned a thread already ?
            if state.2.is_none() {
                // println!("Timer::Launch Thread");
                let waker = cx.waker().clone(); // clone ArcWaker reference
                let timeout = state.0; // copy timeout value
                let ts = self.0.clone(); // clone Arc<T>
                state.2 = Some(thread::spawn(move || {
                    thread::park_timeout(timeout);
                    // println!("Timer::Thread Lapsed");

                    ts.lock()
                        .map(|mut state| {
                            // println!("Timer::Locked & mutate");
                            state.1 = true;
                        })
                        .expect("Mutex poisoned");

                    // println!("Timer::Waker::wake()");
                    waker.wake();
                }));
            }
        }
        // println!("Timer::Not Lapsed");
        Poll::Pending
    }
}
