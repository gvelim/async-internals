
pub trait SimpleFuture {
    type Output;
    fn poll( &mut self, waker: fn(&mut Self) ) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}

pub struct MyTask {
    pub b: bool,
    pub waker: Option<fn(&mut Self)>,
}

impl SimpleFuture for MyTask {
    type Output = bool;

    fn poll(&mut self, waker: fn(&mut Self)) -> Poll<Self::Output> {
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
        self.waker.take().unwrap()(self);
    }
    pub fn test_waker(&mut self) {
        println!("\t\t ... wake, wake !! {}",self.b);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut task = MyTask::new();

        println!("In: {:?}", task.b);
        while !match task.poll(MyTask::test_waker) {
            Poll::Ready(val) => {
                println!("Round {:?}",task.b);
                val
            },
            Poll::Pending => {
                println!("Round {:?}",task.b);
                task.do_something();
                false
            },
        } {
            println!("While: {:?}",task.b);
        }
        assert_eq!(false, true);
    }
}
