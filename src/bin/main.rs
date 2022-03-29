use async_test::*;

fn main() {
    let mut task = MyTask::new();
    task.do_something();

    println!("In: {:?}", task.b);
    while !match task.poll(MyTask::waker) {
        Poll::Ready(val) => {
            print!("R{:?}",task.b);
            val
        },
        Poll::Pending => {
            print!("P{:?}",task.b);
            false
        },
    } {
        print!(",");
    }

}