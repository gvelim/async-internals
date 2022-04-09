use std::sync::Arc;
use futures::task::ArcWake;

pub struct MyWaker;

impl ArcWake for MyWaker {
    fn wake(self: Arc<Self>) {
        println!("\nWooHoo... I woke up!!");
    }
    fn wake_by_ref(_: &Arc<Self>) {
        println!("\nWooHoo... I woke up!!");
    }
}