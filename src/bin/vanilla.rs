use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Waker,Poll},
    time::{Duration, Instant},
};
use std::collections::VecDeque;
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
        println!("\nWooHoo... I woke up!!");
    }
    fn wake_by_ref(_: &Arc<Self>) {
        println!("\nWooHoo... I woke up!!");
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
    // MyTimer::functions can be .awaited
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

struct MyExecutor {
    queue: VecDeque<Arc<MyTask>>,
}

impl MyExecutor {
    fn new() -> MyExecutor {
        MyExecutor { queue: VecDeque::new() }
    }
    fn spawn(&mut self, fut: impl Future<Output=()> + Send + 'static) {
        self.queue.push_back(Arc::new(MyTask::new(fut)));
    }
    fn run(&mut self) {
        while let Some(task) = self.queue.pop_front().take() {
            if task.clone().poll().is_pending() {
                self.queue.push_back(task.clone());
                thread::sleep(Duration::from_millis(10));
                print!(".");
            }
        }
    }
}

async fn wait_timer(lapse: u64) -> &'static str {
    MyTimer::new(lapse).await
}

fn main() {
    let mut exec = MyExecutor::new();

    let now = Instant::now();

    exec.spawn(async {
        println!("F1: {}", wait_timer(3).await);
    });
    exec.spawn(async {
        println!("F2: {}", MyTimer::new(2).await);
    });
    exec.spawn(async {
        println!("F3: {}", MyTimer::new(1).await);
    });

    exec.run();
    println!("finished: {:?}", now.elapsed() )
}