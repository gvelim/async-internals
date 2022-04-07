use std::{
    thread,
    sync::{Arc,Mutex},
    pin::Pin,
    task::Poll,
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

async fn test() {
    thread::spawn(move || {
        println!("spawn thread");
        thread::sleep(std::time::Duration::new(5, 0));
        for i in 1..=10 {
            print!("{},", i);
        }
        println!("thread finished");
    });
}

fn main() {
    let f = async{
        println!("Enter async...");
        test().await;
        println!("Exiting async...");
    };
    let fut = Arc::new(MyTask::new(f));

    while fut.clone().poll().is_pending() {
        print!("z,");
    }

}