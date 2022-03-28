use async_test::*;

fn test_waker() {
    println!("wake, wake");
}

fn main() {
    let mut task = MyTask::new();

    println!("{:?}", task);
    while match task.poll(test_waker) {
        Poll::Ready(_) => {
            println!("Round {:?}",task);
            false
        },
        Poll::Pending => {
            println!("Round {:?}",task);
            task.do_something();
            true
        },
    } {
        println!("Round {:?}",task);
    }
}