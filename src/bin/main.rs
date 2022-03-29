use async_test::*;
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel,Receiver,SyncSender},
    sync::{Arc,Mutex},
    task::{Context},
    time::Duration,
};

struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

fn new_spawn_and_exec() -> (Executor, Spawner) {
    let (task_sender, ready_queue) = sync_channel(10000);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {

    fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks");
    }
}

impl Executor {
    fn run(&self) {

        while let Ok(task) = self.ready_queue.recv() {

            let mut future_slot = task.future.lock().unwrap();

            if let Some(mut future) = future_slot.take() {

                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);

                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }


        }
    }
}

fn main() {
    let (exec, spawn) = new_spawn_and_exec();

    spawn.spawn( async {
        println!("Async Task: Start");
        TimeFuture::new( Duration::new(2,0)).await;
        println!("Done");
    });

    drop(spawn);

    exec.run();

}