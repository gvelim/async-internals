use std::{
    sync::{Arc,Mutex, mpsc::*},
    future::Future,
};

use super::mytask::MyTask;

pub struct MyExecutor<'a> {
    queue: Receiver<Arc<MyTask<'a>>>,
    sender: Option<SyncSender<Arc<MyTask<'a>>>>,
}

impl<'a> MyExecutor<'a> {
    pub fn init() -> MyExecutor<'a> {
        let (sender, queue) = std::sync::mpsc::sync_channel(1000);
        MyExecutor {
            queue,
            sender: Some(sender),
        }
    }

    pub fn spawn(&mut self, f: impl Future<Output = ()> + Send + 'a) {
        let t = MyTask(
            Mutex::new(Box::pin(f)),
            Some(self.sender.as_ref().unwrap().clone()),
        );
        self.sender
            .as_ref()
            .unwrap()
            .send(Arc::new(t))
            .expect("cannot push in the queue");
    }

    pub fn drop_spawner(&mut self) {
        let s = self.sender.take();
        drop(s);
    }

    pub fn run(&mut self) {
        // release our reference to the sender so channel gets dropped once the last Task completes
        // hence causing the queue to terminate and exit the while loop
        self.drop_spawner();
        while let Ok(task) = self.queue.recv() {
            // println!("Exec::Received()");
            task.poll();
        }
    }
}