use async_test::{TimeFuture, executor};
use std::time::Duration;

fn main() {
    let (exec, spawn) = executor::new_spawn_and_exec();

    spawn.spawn( async {
        println!("Async Task: Start");
        TimeFuture::new( Duration::new(2,0)).await;
        println!("Done");
    });

    drop(spawn);

    exec.run();
}