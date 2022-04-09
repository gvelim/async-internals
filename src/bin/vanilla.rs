use rand::prelude::*;
use std::time::{Duration, Instant};
use futures::executor;
use futures::executor::{LocalPool, ThreadPool};
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
            });
    }

    pool.run();

}

fn main() {
    println!("Running futures using myexecutor");
    run_myexec();
    println!("Running futures using futures::executor");
    run_localexec();
}