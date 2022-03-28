
pub trait SimpleFuture {
    type Output;
    fn poll( &mut self, waker: fn() ) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
#[derive(Debug)]
pub struct MyTask {
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
    pub fn new() -> MyTask {
        MyTask { b: false, waker: None }
    }
    pub fn do_something(&mut self) {
        self.b = true;
        self.waker.take().unwrap()();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

        assert_eq!(true, true);
    }
}
