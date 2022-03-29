use std::sync::mpsc::*;
use std::time::Duration;

pub trait SimpleFuture {
    type Output;
    fn poll( &mut self, waker: fn(&Sender<bool>) ) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}

pub struct MyTask {
    pub b: bool,
    waker: Option<fn(&Sender<bool>)>,
    rx: Receiver<bool>,
    tx: Sender<bool>,
}

impl SimpleFuture for MyTask {
    type Output = bool;

    fn poll(&mut self, waker: fn(&Sender<bool>)) -> Poll<Self::Output> {
        return if self.rx.recv_timeout(Duration::from_nanos(1)) == Ok(true) {
            self.b = true;
            Poll::Ready(self.b)
        } else {
            self.waker = Some(waker);
            Poll::Pending
        }
    }
}

impl MyTask {

    pub fn new() -> MyTask {
        use std::sync::mpsc::*;
        let (tx,rx) = channel::<bool>();
        MyTask { b: false, waker: None, rx, tx }
    }

    pub fn waker(tx: &std::sync::mpsc::Sender<bool>) {
        println!("\t\t ... wake, wake !!");
        tx.send(true);
    }

    pub fn do_something(&self) {
        use std::{thread::*, time::*};

        let tx = self.tx.clone();
        let waker = MyTask::waker;
        spawn( move || {
            println!("\tThread started");
            sleep( Duration::from_secs(1));
            println!("\tThread lapsed");
            waker(&tx);
        });
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1+1, 2);
    }
}
