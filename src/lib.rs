
trait SimpleFuture {
    type Output;
    fn poll( &mut self, waker: fn() ) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}

struct MyTask {
    b: bool,
    waker: Option<fn()>,
}

impl SimpleFuture for MyTask {
    type Output = bool;

    fn poll(&mut self, waker: fn()) -> Poll<Self::Output> {
        return if self.b == true {
            Poll::Ready(true)
        } else {
            self.waker = Some(waker);
            Poll::Pending
        }
    }
}

impl MyTask {
    fn new() -> MyTask {
        MyTask { b: false, waker: None }
    }
    fn do_something(&mut self) {
        self.b = true;
        self.waker.take();
    }
}


#[cfg(test)]
mod tests {
    use crate::{MyTask};

    #[test]
    fn it_works() {
        let mut task = MyTask::new();

        task.do_something();

        assert_eq!(true, true);
    }
}
