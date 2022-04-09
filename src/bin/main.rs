use async_test::rustfuture::{TimeFuture,executor};
use std::time::Duration;
use rand::prelude::*;

fn main() {
    let (exec, spawn) = executor::new_spawn_and_exec();

    for i in 0..10 {
        spawn.spawn( async move {
            println!("Async Task {i}: Start");
            let d: u64 = thread_rng().gen_range(1..10);
            TimeFuture::new( Duration::new(d,0)).await;
            println!("Done");
        });
    }

    drop(spawn);

    exec.run();
}