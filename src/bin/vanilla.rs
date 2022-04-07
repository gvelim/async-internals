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
    // has to be wrapped in Mutex so MyTask inherits the Send trait
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
    // since MyTimer impl Future
    // MyTimer::functions can be .await'ed
    // causing MyTimer::poll() to be called
    fn new(lapse: u64) -> MyTimer {

        // initialise Timer struct
        let mt = MyTimer {
            lapsed: Arc::new(Mutex::new( false)),
            waker: Arc::new(Mutex::new( None)),
        };

        // extract pointer references
        let ref_lapsed = mt.lapsed.clone();
        let ref_waker = mt.waker.clone();

        // spawn timer
        thread::spawn( move || {
            thread::sleep( Duration::new(lapse,0));
            // lock attr and change value
            let mut lapsed = ref_lapsed.lock().unwrap();
            *lapsed = true;
            // lock attr and callback using stored *fn()
            let mut waker = ref_waker.lock().unwrap();
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