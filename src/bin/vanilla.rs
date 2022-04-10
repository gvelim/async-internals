use rand::prelude::*;
use std::time::Instant;
use futures::executor;
use futures::executor::{block_on};
use futures::task::SpawnExt;
use async_test::myfuture::{
    MyTimer,
    myexecutor::MyExecutor
};

async fn wait_timer(lapse: u64) -> &'static str {
    MyTimer::new(lapse).await
}

fn run_myexec() {

    let mut exec = MyExecutor::new();

    let now = Instant::now();

    for i in (1..=10).rev() {
        let d: u64 = thread_rng().gen_range(1..=10);
        exec.spawn(async move {
            println!("F{}: {}", i, wait_timer(d).await );
        });
    }

    exec.run();
    println!("Total time: {:.2?}", now.elapsed() )
}

fn run_localexec() {
    let mut pool = executor::LocalPool::new();

    for i in (1..=10).rev() {
        let d: u64 = thread_rng().gen_range(1..=10);
        pool.spawner()
            .spawn(async move {
                println!("F{}:{}", i, MyTimer::new(d).await);
            }).unwrap();
    }
    pool.run();
}

fn run_threadpool_exec() {
    let pool = executor::ThreadPool::new().expect("Error: cannot initiate pool");
    let mut hnd = Vec::new();

    for i in (1..=10).rev() {
        let d: u64 = thread_rng().gen_range(1..=10);
        let h = pool.spawn_with_handle(async move {
                println!("F{}:{}", i, MyTimer::new(d).await);
            }).unwrap();
        hnd.push(h);
    }
    for h in hnd {
        print!("{:?} = ", h);
        println!("{:?}", block_on(h));
    }
}

fn main() {
    println!("MyExecutor:");
    run_myexec();
    println!("LocalThread:");
    run_localexec();
    println!("ThreadPool:");
    run_threadpool_exec();
}