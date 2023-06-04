use core::{pin::Pin, task::*, future::*};
use std::sync::Arc;

use futures::{FutureExt, task::waker_ref};

struct MyFuture(i32);
impl Future for MyFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 10 {
            println!("Poll::Done {}",self.0);
            return Poll::Ready(self.0)
        } else {
            // do some work
            self.0 += 1;
            println!("MyFuture::Poll() - Checking({}) ",self.0);
            // A Waker is a handle for waking up a task by notifying its executor that it is ready to be run.
            // call byref as otherwise we'll consume the Waker
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn my_async_fn(i: i32) -> impl Future<Output=i32> {
   println!("my_async_fn() - Called with {i}");
   MyFuture(i)
}

async fn an_async_fn(i: i32) -> i32 {
    println!("an_async_fn()");
    i+1
}

fn main() {
    use futures::executor::block_on;

    let f1 = my_async_fn(0);
    let out = block_on(f1);
    println!("Async completed! {:?}", out);
}


/// Demonstrating the fundamentals for an executor
/// - A task that holds a Mutex<BoxedFuture> ; Task needs Mutex so it inherits the Sync in addition to Sent, hence it is +Sync +Sent
/// - A task that Wakes; implements the Waker trait
/// Explain the executor hold and Arc<Task>, in order to
/// (a) create the task's context and poll the future
/// (b) while polling, exclusively lock the task from being awakened by the future
///
#[test]
fn test_simple_executor() {
    use std::sync::Arc;
    use futures::future::BoxFuture;
    use futures::task::{ArcWake};
    use std::sync::Mutex;
    
    // Define a Task that holds a Boxed Future Object on the heap
    struct MyTask<'a>(Mutex<BoxFuture<'a, i32>>);
    // Make it a waker
    impl ArcWake for MyTask<'_> {
        fn wake_by_ref(arc_self: &Arc<Self>) { 
            print!("Wake {:p}", arc_self);
        }
    }

    // Capture Future from stack and
    let f = my_async_fn(5);
    // Construct a Task that moves the future on the heap and pins it
    // We wrap the future in Mutex to ensure Task = Sent + Sync
    // We place the task in a Atomic Reference Counting pointer
    let t = Arc::new(MyTask( Mutex::new(Box::pin(f))));

    // Construct the Task's Context using Task's ArcWake trait implementation
    let wk = waker_ref(&t);
    let mut ctx = Context::from_waker(&wk);

    let _n = loop {
        print!("Manually Poll->");
        // access the task : t.0 (Arc<> dereferences here)
        // Lock access to the task: lock().unwrap() (we infer no poisoning here)
        // get mutable access into the Boxed future
        // call Poll() on the pinned future
        match t.0.lock().unwrap().as_mut().poll(&mut ctx) {
            Poll::Pending => println!("Not ready yet ->"),
            Poll::Ready(n) => {
                println!("Finished = {n}");
                break n;
            }
        }
    };
}

/// Demonstrating a simple manual polling by constructing
/// - A task that holds a Future object on the heap, hence Boxed and Pinned + Send 
/// - A Dummy Waker & Context
/// Explain how MyFuture::Poll is calling the Waker to signal the executor to Poll() again
#[test]
fn test_manually_poll_future() {
    use std::sync::Arc;
    use std::task::Wake;
    use futures::future::BoxFuture;
    
    // Define a Task that holds a Boxed Future Object on the heap
    struct MyTask<'a>(BoxFuture<'a, i32>);
    // Define a dummy Waker struct for now without an associated feature/task
    struct MyWake;
    // Make it a waker
    impl Wake for MyWake {
        fn wake(self: Arc<Self>) {
            println!("Wake()");
        }
        fn wake_by_ref(self: &Arc<Self>) {
            print!("Wake_by_ref() - Poll me!");
        }
    }

    // Capture Future from stack and Construct a Task that moves the future on the heap
    let f = my_async_fn(5);
    let mut t = MyTask( Box::pin(f));

    // Construct Waker & Context
    let wk = Waker::from(Arc::new(MyWake));
    let mut ctx = Context::from_waker(&wk);

    let _n = loop {
        print!("Manually Poll->");
        let f = t.0.as_mut();
        match f.poll(&mut ctx) {
            Poll::Pending => println!("Not ready yet ->"),
            Poll::Ready(n) => {
                println!("Finished = {n}");
                break n;
            }
        }
    };

}

/// Demonstrate mutliple futures executing concurently
/// Explain here that Join() returns a root-future that contains all other futures, 
/// hence you have a tree of futures that is hieratchically polled
/// Explain you can shape sequencial and concurent execution paths to meet your needs
#[test]
fn test_join_up_futures() {
    use futures::executor::block_on;
    use futures::future::join;

    let f1 = my_async_fn(0);
    let f2 = an_async_fn(5);
    let f3 = my_async_fn(5);
    let out = block_on(
        join(f1, join(f2, f3))
    );
    println!("Async completed! {:?}", out);
}

/// Demonstrate how we do the same as below but with LocalThreadPool (N:M) threads
/// Explain that you can plug and play any runtime/executor suitable to your solution needs
#[test]
fn test_run_future_on_local_pool() {
    use futures::executor::*;
    use futures::task::LocalSpawnExt;

    let mut pool = LocalPool::new();
    pool.spawner().spawn_local( async {
        my_async_fn(0).await;
        my_async_fn(5).await;
        an_async_fn(5).await;
    }).expect("");
    pool.run();
}

/// Demonstrate async block relationship to Future Objects
/// Explain async{} blocks return a future that need to be polled. 
/// Explain that futures are executed hierachically with root sequencially f1,f2,f3 futures 
#[test]
fn test_block_on_future_with_async() {
    use futures::executor::block_on;
    let root = async {
        my_async_fn(0).await;
        my_async_fn(5).await;
        an_async_fn(5).await;
    };
    let out = block_on(root);
    println!("Async completed! {:?}", out);
}

/// Demonstrate how we can get our Future Object executed by block_on()
/// Explain that the block_on() never calls back again once the ctx.wake() is removed from within MyFuture::poll()
/// Reason: We never notify the executor to call us back!!
/// Futures if not ready once called, must notify the executor when "ready to be called back"
/// Explain Futures unless Polled, occupy now resources
/// Explain Futures are desugared into state machines, yelding control and continuing executing from where they last paused
#[test]
fn test_block_on_future() {
    use futures::executor::block_on;

    let f1 = my_async_fn(0);
    let out = block_on(f1);
    println!("Async completed! {:?}", out);
}