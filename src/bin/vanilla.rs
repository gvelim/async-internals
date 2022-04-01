use std::{
    thread,
    sync::Arc,
};
use futures::{
    Future,
    task::{Context, ArcWake, waker}
};

struct MyWaker;

impl ArcWake for MyWaker {
    fn wake(self: Arc<Self>) {
        println!("WooHoo... I woke up!!");
    }
    fn wake_by_ref(_: &Arc<Self>) {
        println!("WooHoo... I woke up!!");
    }
}

fn main() {
    let fut = async {
        println!("Enter async...");
        thread::sleep( std::time::Duration::new(5,0));
        for i in 1..=10 {
            print!("{},",i);
        }
        println!("Exiting async...");
    };

    let mut fut = Box::pin(fut);

    let waker = waker( Arc::new(MyWaker));
    let mut cx = Context::from_waker( &waker);

    while fut.as_mut().poll(&mut cx).is_pending() {
        print!("z,");
    }

}