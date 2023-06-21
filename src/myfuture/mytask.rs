use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake, Context},
};
use std::sync::{
    mpsc::SyncSender,
    Arc, Mutex,
};
use std::thread;

// Define a Task that holds a Boxed Future Object on the heap
pub struct MyTask<'a> (
    pub Mutex<BoxFuture<'a, ()>>, 
    pub Option<SyncSender<Arc<Self>>>
);

impl<'a> MyTask<'a> {
    // Poll can be called only when MyTask is wrapped in an Arc<T>
    pub fn poll(self: &Arc<Self>) {
        let waker = waker_ref(self);
        let ctx = &mut Context::from_waker(&waker);

        self.0.lock().unwrap().as_mut().poll(ctx).is_pending();
    }
    // Schedule can be called only when MyTask is wrapped in an Arc<T>
    pub fn schedule(self: &Arc<Self>) {
        self.1
            .as_ref()
            .map(|s| {
                s.send(self.clone())
                    .expect("MyTask::schedule() - Cannot queue task")
            })
            .expect("Task::schedule() - Error scheduling task");
    }
}

// Make it a waker
impl ArcWake for MyTask<'_> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!("Wake from {:?}", thread::current().id());
        arc_self.schedule();
    }
}
    
