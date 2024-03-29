use std::sync::Arc;
use rand::{Rng, thread_rng};
use tokio::time;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;

struct Fork;

struct Philosopher {
    name: String,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    thoughts: Sender<String>
}

impl Philosopher {
    async fn think(&self) {
        self.thoughts
            .send(format!("Eureka! {} has a new idea!", &self.name)).await
            .unwrap();
    }

    async fn eat(&self) {
        // Pick up forks...
        let _lf = self.left_fork.lock().await;
        // print!("{} picked left fork...",self.name);
        let _rf = self.right_fork.lock().await;
        // print!("{} picked right fork...",self.name);
        println!("{} is eating... {:?}", &self.name, std::thread::current().id() );
        let delay = thread_rng().gen_range(1..10);
        time::sleep( time::Duration::from_millis(delay*100) ).await;
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras", "George V", "Satre"];

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {    
    
    async fn app() {
        // Create forks
        let forks = PHILOSOPHERS
            .iter()
            .map(|_| Arc::new(Mutex::new(Fork)))
            .collect::<Vec<_>>();
        
        let (tx,mut rx) = mpsc::channel::<String>(2);

        PHILOSOPHERS.iter().enumerate()
            // Create philosophers
            .map(|(i,philosopher)| {
                Philosopher {
                    name: philosopher.to_string(),
                    left_fork: forks[i].clone(),
                    right_fork: forks[ (i+1) % PHILOSOPHERS.len() ].clone(),
                    thoughts: tx.clone()
                }
            })
            // Make them think and eat
            .for_each(|philosopher| {
                tokio::task::spawn_local(async move {
                    for _ in 0..10 {
                        philosopher.eat().await;
                        philosopher.think().await;
                    }
                });
            });
        
        drop(tx);
        // Output their thoughts
        while let Some(thought) = rx.recv().await {
            println!("{}",thought);
        }
    }
    
    // force tasks to run against the local thread
    let ls = tokio::task::LocalSet::new();
    ls.run_until(app()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_thread_pool() {    
    
    // Create forks
    let forks = PHILOSOPHERS.iter()
        .map(|_| Arc::new(Mutex::new(Fork)))
        .collect::<Vec<_>>();
    
    let (tx,mut rx) = mpsc::channel::<String>(3);
    PHILOSOPHERS.iter().enumerate()
        // Create philosophers
        .map(|(i,philosopher)| {
            Philosopher {
                name: philosopher.to_string(),
                left_fork: forks[i].clone(),
                right_fork: forks[ (i+1) % PHILOSOPHERS.len() ].clone(),
                thoughts: tx.clone()
            }
        })
         // Make them think and eat
        .for_each(|philosopher| {
            tokio::spawn(async move {
                for _ in 0..10 {
                    philosopher.eat().await;
                    philosopher.think().await;
                }
            });
        });
    
    drop(tx);
    // Output their thoughts
    while let Some(thought) = rx.recv().await {
        println!("{}",thought);
    }

}