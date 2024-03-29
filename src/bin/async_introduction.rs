use core::{future::*, pin::*, task::*};

struct MyFuture(i32);
impl Future for MyFuture {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 0 {
            println!("Poll::Done {}", self.0);
            Poll::Ready(self.0)
        } else {
            // do some work
            self.0 -= 1;
            println!("MyFuture::Poll() - Checking({:?}) ", self.0);
            // A Waker is a handle for waking up a task by notifying its executor that it is ready to be run.
            // call byref as otherwise we'll consume the Waker
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn my_async_fn(i: i32) -> impl Future<Output = i32> {
    println!("my_async_fn() - Called with {i}");
    MyFuture(i)
}

async fn an_async_fn(i: u32) -> u32 {
    println!("an_async_fn()");
    i + 1
}

fn main() {
    use futures::executor::block_on;

    let f1 = my_async_fn(5);
    println!("Async completed! {:?}", block_on(f1));
    let f2 = async { an_async_fn(10).await };
    println!("Async completed! {:?}", block_on(f2));
}

/// Demonstrate an improved version of the basic Executor that Polls until all futures have returned
/// - Futures are now having their own thread that is signalling awake() on task completion; wake() removed from within poll()
/// - Executor run() method now uses a thread message queue that awaits to receive and process atomic tasks references
/// - Executor spawn() method now pushes atomic task references onto the message queue for execution
/// - Waker() now pushes atomic references of awaken tasks down the message queue for execution
///
#[test]
fn test_future_callback() {
    use futures::{
        future::BoxFuture,
        task::{waker_ref, ArcWake},
    };
    use std::sync::{
        mpsc::{Receiver, SyncSender},
        Arc, Mutex,
    };
    use std::time::*;
    use std::{thread, thread::JoinHandle};

    type Data = (Duration, bool, Option<JoinHandle<()>>);
    struct Timer(Arc<Mutex<Data>>);
    impl Timer {
        fn start(timeout: Duration) -> impl Future<Output = ()> {
            println!("Timer::start()");
            Timer(Arc::new(Mutex::new((timeout, false, None))))
        }
    }
    impl Future for Timer {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut state = self.0.lock().unwrap();
            if state.1 {
                println!("Timer::Lapsed");
                return Poll::Ready(());
            } else {
                // have we spawned a thread already ?
                if state.2.is_none() {
                    println!("Timer::Launch Thread");
                    let waker = cx.waker().clone(); // clone ArcWaker reference
                    let timeout = state.0; // copy timeout value
                    let ts = self.0.clone(); // clone Arc<T>
                    state.2 = Some(thread::spawn(move || {
                        thread::park_timeout(timeout);
                        println!("Timer::Thread Lapsed");

                        ts.lock()
                            .map(|mut state| {
                                println!("Timer::Locked & mutate");
                                state.1 = true;
                            })
                            .expect("Mutex poisoned");

                        println!("Timer::Waker::wake()");
                        waker.wake();
                    }));
                }
            }
            println!("Timer::Not Lapsed");
            Poll::Pending
        }
    }

    // Define a Task that holds a Boxed Future Object on the heap
    struct MyTask<'a>(Mutex<BoxFuture<'a, ()>>, Option<SyncSender<Arc<Self>>>);
    impl<'a> MyTask<'a> {
        // Poll can be called only when MyTask is wrapped in an Arc<T>
        fn poll(self: &Arc<Self>) {
            let waker = waker_ref(self);
            let ctx = &mut Context::from_waker(&waker);

            self.0.lock().unwrap().as_mut().poll(ctx).is_pending();
        }
        // Schedule can be called only when MyTask is wrapped in an Arc<T>
        fn schedule(self: &Arc<Self>) {
            self.1
                .as_ref()
                .map(|s| {
                    s.send(self.clone())
                        .expect("MyTask::schedule() - Cannot queue task")
                })
                .expect("Task::schedule() - Error scheduling");
        }
    }
    // Make it a waker
    impl ArcWake for MyTask<'_> {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            println!("Wake from {:?}", thread::current().id());
            arc_self.schedule();
        }
    }
    struct Executor<'a> {
        queue: Receiver<Arc<MyTask<'a>>>,
        sender: Option<SyncSender<Arc<MyTask<'a>>>>,
    }
    impl<'a> Executor<'a> {
        fn init() -> Executor<'a> {
            let (sender, queue) = std::sync::mpsc::sync_channel(1000);
            Executor {
                queue,
                sender: Some(sender),
            }
        }
        fn spawn(&mut self, f: impl Future<Output = ()> + Send + 'a) {
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
        fn drop_spawner(&mut self) {
            let s = self.sender.take();
            drop(s);
        }
        fn run(&mut self) {
            // release our reference to the sender so channel gets dropped once the last Task completes
            // hence causing the queue to terminate and exit the while loop
            self.drop_spawner();
            while let Ok(task) = self.queue.recv() {
                println!("Exec::Received()");
                task.poll();
            }
        }
    }

    let mut exec = Executor::init();

    for i in 1..2 {
        exec.spawn(async move { Timer::start(Duration::from_millis(i * 1000)).await });
    }
    exec.spawn(async {
        Timer::start(Duration::from_millis(3000)).await;
        println!("Finished: {}", my_async_fn(5).await);
        println!("Finished: {}", my_async_fn(10).await);
    });
    exec.spawn(async {
        println!("Finished: {}", my_async_fn(3).await);
        println!("Finished: {}", an_async_fn(1).await);
    });

    exec.run();
}

/// Demonstrate a very basic Executor that Polls until all futures have returned
/// - Holds a vector of atomic references to Tasks
/// - Uses a spawn() method to wrap a future into a trait object and push its atomic reference onto the vector queue
/// - Uses a run() method to iterate over the queue and call the Poll() method from each task
/// Limitations:
/// The executor doesn't make use of waker() to be notified as a result it loops over and over again until all futures complete
/// The futures used do not put the executor into sleep since they call wake() within poll()
#[test]
fn test_simple_executor() {
    use std::sync::{Arc, Mutex};
    // use std::task::*;
    use futures::future::BoxFuture;
    use futures::task::{waker_ref, ArcWake};
    use std::collections::VecDeque;

    // Define a Task that holds a Boxed Future Object on the heap
    struct MyTask<'a>(Mutex<BoxFuture<'a, ()>>, usize);
    // Make it a waker
    impl ArcWake for MyTask<'_> {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            print!("Waker TaskID:({:02})->", arc_self.1);
        }
    }

    struct MyExecutor<'a> {
        tasks: VecDeque<Arc<MyTask<'a>>>,
    }
    impl MyExecutor<'_> {
        fn spawn(&mut self, f: impl Future<Output = ()> + Send + 'static) {
            self.tasks
                .push_back(Arc::new(MyTask(Mutex::new(Box::pin(f)), self.tasks.len())))
        }
        fn run(&mut self) {
            while let Some(task) = self.tasks.pop_front() {
                let waker = waker_ref(&task);
                let mut ctx = Context::from_waker(&waker);

                if task.0.lock().unwrap().as_mut().poll(&mut ctx).is_pending() {
                    self.tasks.push_back(task)
                }
            }
        }
    }

    let mut exec = MyExecutor {
        tasks: VecDeque::new(),
    };

    exec.spawn(async {
        println!("Finished: {}", my_async_fn(10).await);
    });
    exec.spawn(async {
        println!("Finished: {}", my_async_fn(5).await);
        println!("Finished: {}", my_async_fn(3).await);
    });
    exec.spawn(async {
        println!("Finished: {}", my_async_fn(1).await);
    });

    exec.run();
}
/// Demonstrating the fundamentals for an executor
/// - A task that holds a Mutex<BoxedFuture> ; Task needs Mutex so it inherits the 'Sync' in addition to 'Sent' traits
/// - A task that Wakes by implementing the 'ArcWaker' trait
/// Explain the executor holds onto the Arc<Task>, in order to
/// (a) create the task's context from 'Arc<impl ArcWaker>'; derived from 'Arc<MyTask>'
/// (b) exclusively 'lock()' the Task (which is 'Sync') and get access to the boxed future (which is 'Send')
/// (c) 'poll()' the future with the task's context
/// (d) observe the future's use of the 'waker_by_ref()' callback aiming to trigger polling again
///
#[test]
fn test_simple_task_waker() {
    use futures::FutureExt;
    use std::sync::{Arc, Mutex};
    use std::task::Wake;

    // Define a Task that holds a Boxed Future Object on the heap
    // Wrap Box in Mutex<T> since we need to mutate against a immutable Task reference, hence Arc<Mutex<T>>
    // Future<T> + Send is needed for the Arc<T>
    // Future<T> + Unpin is needed for the poll_unpin()
    struct MyTask<T>(Mutex<Box<dyn Future<Output = T> + Unpin + Send>>);
    // Make MyTask a Waker (trait object)
    // ArcWake<T> must have T + Send + Sync; Send from Mutex<T>, Sync from Arc<T>
    impl<T> Wake for MyTask<T> {
        fn wake(self: Arc<Self>) {
            print!("Waker Location:({:p})->", &self.0);
        }
    }

    // Capture Future from stack and
    let f = my_async_fn(5);
    // Construct a Task that moves the future on the heap and pins it
    // We wrap the future in Mutex to ensure Task = Sync
    // We place the task in a Atomic Reference Counting pointer
    // as the Waker will be constructed from Arc<impl ArcWake> == Arc<MyTask>
    let t = Arc::new(MyTask(Mutex::new(Box::new(f))));

    // Extract Waker from Task's ArcWake trait implementation
    // Creates a reference to a Waker from a reference to Arc<impl Wake>, which is why we have to wrap our task in Arc<T>.
    let wk = t.clone().into();
    // Construct the Task's Context
    let mut ctx = Context::from_waker(&wk);

    let _n = loop {
        print!("Poll Task:({:p})->", &t.0);
        // access the task : t.0 (Arc<> dereferences here)
        // Lock access to the task: lock().unwrap() (we infer no poisoning here)
        // get mutable access into the Boxed future
        // call Poll() on the pinned future
        match t.0.lock().unwrap().as_mut().poll_unpin(&mut ctx) {
            Poll::Pending => println!("Not ready yet ->"),
            Poll::Ready(n) => {
                println!("Finished = {n}");
                break n;
            }
        }
    };
}

/// Demonstrating a simple manual polling by
/// 1. constructing a Dummy Waker & Context
/// 2. use the context to future poll() method
/// Explain how MyFuture::Poll is calling the Waker to signal the executor to Poll() again
/// Explain limitation that
/// 1. task context, hasn't got much context in it
/// 2. future sits on caller's stack hence not a realistic case
#[test]
fn test_manually_poll_future() {
    use std::sync::Arc;
    use std::task::Wake;
    // use futures::task::noop_waker;

    // Define a dummy Waker struct for now without an associated feature/task
    struct MyWake;
    // Make it a Waker
    impl Wake for MyWake {
        fn wake(self: Arc<Self>) {
            println!("Wake() - Poll again!");
        }
    }

    // Capture Future from stack and Construct a Task that moves the future on the heap
    let f = async { my_async_fn(5).await };

    // Construct a dummy Waker & Context
    // Waker implements From trait on Arc<Wake>
    let wk = Arc::new(MyWake).into();
    // let wk = noop_waker();
    let mut ctx = Context::from_waker(&wk);

    let mut f = pin!(f);
    let _n = loop {
        print!("Exec::Poll->");
        match f.as_mut().poll(&mut ctx) {
            Poll::Pending => println!("Exec::Pending ->"),
            Poll::Ready(n) => {
                println!("Exec::Ready = {n}");
                break n;
            }
        }
    };
}

/// Demonstrate multiple futures executing concurrently
/// Explain here that Join() returns a root-future that contains all other futures,
/// hence you have a tree of futures that is hierarchically polled
/// Explain you can shape sequential and concurrent execution paths to meet your needs
#[test]
fn test_join_up_futures() {
    use futures::executor::block_on;
    use futures::future::join;

    let f1 = my_async_fn(10);
    let f2 = an_async_fn(5);
    let f3 = my_async_fn(5);
    let out = block_on(join(f1, join(f2, f3)));
    println!("Async completed! {:?}", out);
}

/// Demonstrate how we do the same as below but with LocalThreadPool (N:M) threads
/// Explain that you can plug and play any runtime/executor suitable to your solution needs
#[test]
fn test_run_future_on_local_pool() {
    use futures::executor::*;
    use futures::task::LocalSpawnExt;

    let mut pool = LocalPool::new();
    pool.spawner()
        .spawn_local(async {
            my_async_fn(10).await;
            my_async_fn(5).await;
            an_async_fn(5).await;
        })
        .expect("");
    pool.run();
}

/// Demonstrate async block relationship to Future Objects
/// Explain async{} blocks return a future that need to be polled.
/// Explain that futures are executed hierarchically with root sequentially f1,f2,f3 futures
#[test]
fn test_block_on_future_with_async() {
    use futures::executor::block_on;
    let root = async {
        my_async_fn(10).await;
        my_async_fn(5).await;
        an_async_fn(5).await;
    };
    println!("Async completed! {:?}", block_on(root) );
}

/// Demonstrate how we can get our Future Object executed by block_on()
/// Explain that the block_on() never calls back again once the ctx.wake() is removed from within MyFuture::poll()
/// Reason: We never notify the executor to call us back!!
/// Futures if not ready once called, must notify the executor when "ready to be called back"
/// Explain Futures unless Polled, occupy now resources
/// Explain Futures are desugared into state machines, yielding control and continuing executing from where they last paused
#[test]
fn test_block_on_future() {
    use futures::executor::block_on;

    let f1 = my_async_fn(5);
    let out = block_on(f1);
    println!("Async completed! {:?}", out);
}