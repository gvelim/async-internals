use std::{
    future::Future,
    pin::Pin,
    sync::{Arc,Mutex},
    task::{Context,Poll,Waker},
    thread,
    time::Duration,
};

pub mod executor;

pub struct TimeFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimeFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimeFuture {

    pub fn new(duration: Duration) -> TimeFuture {
        let shared_state = Arc::new(
            Mutex::new( SharedState {
                completed: false,
                waker: None,
            })
        );

        let thread_shared_state = shared_state.clone();
        thread::spawn( move || {
            thread::sleep(duration);
            let mut ss = thread_shared_state.lock().unwrap();
            ss.completed = true;
            if let Some(waker) = ss.waker.take() {
                waker.wake()
            }
        });

        TimeFuture { shared_state }
    }
}