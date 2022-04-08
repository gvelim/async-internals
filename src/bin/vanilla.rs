use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::{Waker,Poll},
    time::{Duration, Instant},
    collections::vec_deque::VecDeque,
};
use futures::{
    Future,
    task::{Context, ArcWake},
    task
};

struct MyWaker;

impl ArcWake for MyWaker {
    fn wake(self: Arc<Self>) {
        println!("\nWooHoo... I woke up!!");
    }
    fn wake_by_ref(_: &Arc<Self>) {
        println!("\nWooHoo... I woke up!!");
    }
}

struct MyTask {
    // has to be wrapped in Mutex so MyTask inherits the Send trait
    fut : Mutex<Pin<Box<dyn Future<Output=()> + Send>>>,
    waker: Arc<MyWaker>,
    lapsed: std::time::Instant
}

impl MyTask {
    fn new(fut: impl Future<Output=()> + Send + 'static) -> MyTask {
        MyTask {
            fut: Mutex::new(Box::pin(fut)),
            waker: Arc::new(MyWaker),
            lapsed: std::time::Instant::now()
        }
    }
    fn poll(&self) -> Poll<()> {

        let waker = task::waker(self.waker.clone());
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
    queue: VecDeque<MyTask>,
}

impl MyExecutor {
    fn new() -> MyExecutor {
        MyExecutor { queue: VecDeque::new() }
    }
    fn spawn(&mut self, fut: impl Future<Output=()> + Send + 'static) {
        self.queue.push_back(MyTask::new(fut));
    }
    fn run(&mut self) {
        while let Some(task) = self.queue.pop_front().take() {
            if task.poll().is_pending() {
                self.queue.push_back(task);
                thread::sleep(Duration::from_millis(10));
                print!(".");
            } else {
                println!("Task took {:.2?}\"", task.lapsed.elapsed());
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

    for i in (1..=10).rev() {
        exec.spawn(async move {
            println!("F{}: {}", i, MyTimer::new(i).await);
        });
    }

    exec.run();
    println!("Total time: {:.2?}", now.elapsed() )
}