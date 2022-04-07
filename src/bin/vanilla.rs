use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Waker,Poll},
    time::{Duration, Instant},
};
use futures::{
    Future,
    task::{Context, ArcWake},
    task
};


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
    waker: Arc<Mutex<Option<Waker>>>,
}
impl MyTimer {
    fn new(lapse: u64) -> MyTimer {
        let mt = MyTimer {
            lapsed: Arc::new( Mutex::new( false)),
            waker: Arc::new(Mutex::new( None)),
        };
        let thread_lapsed = mt.lapsed.clone();
        let thread_waker = mt.waker.clone();

        thread::spawn( move || {
            thread::sleep( Duration::new(lapse,0));
            let mut lapsed = thread_lapsed.lock().unwrap();
            *lapsed = true;
            let mut waker = thread_waker.lock().unwrap();
            match waker.take() {
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
        let lapsed = self.lapsed.lock().unwrap();
        if *lapsed == true {
            Poll::Ready("done")
        } else {
            let mut waker = self.waker.lock().unwrap();
            waker.replace( cx.waker().clone());
            Poll::Pending
        }
    }
}

async fn wait_timer() -> &'static str {
    MyTimer::new(3).await
}

fn main() {
    let now = Instant::now();

    let task = Arc::new(MyTask::new(
        async {
            println!("Future: {}", wait_timer().await);
        }
    ));

    while task.clone().poll().is_pending() {
        thread::sleep(Duration::from_millis(10));
        print!(".");
    }
    println!("finished: {:?}", now.elapsed() )
}