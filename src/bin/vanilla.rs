use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::Poll,
};
use std::ops::Deref;
use std::time::{Duration, Instant};
use futures::{
    Future,
    task::{Context, ArcWake},
    task
};
use crate::task::noop_waker;


struct MyTask {
    fut : Mutex<Pin<Box<dyn Future<Output=()> + Send>>>,
}

impl ArcWake for MyTask {
    fn wake(self: Arc<Self>) {
        println!("WooHoo... I woke up!!");
    }
    fn wake_by_ref(_: &Arc<Self>) {
        println!("WooHoo... I woke up!!");
    }
}

impl MyTask {
    fn new(fut: impl Future<Output=()> + Send + 'static) -> MyTask {
        MyTask {
            fut: Mutex::new(Box::pin(fut))
        }
    }
    fn poll(self: Arc<Self>) -> Poll<()> {

        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker( &waker);

        let mut f = self.fut.lock().unwrap();
        f.as_mut().poll(&mut cx)
    }
}

struct MyTimer {
    lapsed: Arc<Mutex<bool>>,
}

impl MyTimer {
    fn new(lapse: u64) -> MyTimer {
        let mt = MyTimer {
            lapsed: Arc::new( Mutex::new( false)),
        };
        let thread_lapsed = mt.lapsed.clone();
        thread::spawn( move || {
            thread::sleep( Duration::new(lapse,0));
            let mut lapsed = thread_lapsed.lock().unwrap();
            *lapsed = true;
        });
        mt
    }
}

impl Future for MyTimer {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let lapsed = self.lapsed.lock().unwrap();
        if *lapsed == true {
            Poll::Ready("done")
        } else {
            Poll::Pending
        }
    }
}

fn main() {
    let now = Instant::now();

    let f = async {
        let mt = MyTimer::new(3);
        println!("Future: {}",mt.await);
    };

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut f = Box::pin(f);
    while f.as_mut().poll(&mut cx).is_pending() {
        thread::sleep(Duration::from_millis(10));
        print!(".");
    }
    println!("finished: {:?}", now.elapsed() )
}