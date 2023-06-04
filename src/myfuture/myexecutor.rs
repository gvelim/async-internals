use std::{
    thread,
    sync::Arc,
    future::Future,
    time::Duration,
    collections::vec_deque::VecDeque,
};

use crate::myfuture::mytask::MyTask;

pub struct MyExecutor {
    queue: VecDeque<MyTask>,
}

impl Default for MyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MyExecutor {
    pub fn new() -> MyExecutor {
        MyExecutor { queue: VecDeque::new() }
    }
    pub fn spawn(&mut self, fut: impl Future<Output=()> + Send + 'static) {
        self.queue.push_back(MyTask::new(fut));
    }
    pub fn run(&mut self) {
        while let Some(task) = self.queue.pop_front().take() {
            if task.poll().is_pending() {
                self.queue.push_back(task);
                thread::sleep(Duration::from_millis(10));
                print!(".");
            } else {
                println!("Task completed in {:.2?}; waker() RefCount: {}", task.lapsed.elapsed(), Arc::strong_count(&task.waker));
            }
        }
    }
}