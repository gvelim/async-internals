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
        print!("{} picked left fork...",self.name);
        let _rf = self.right_fork.lock().await;
        print!("{} picked right fork...",self.name);
        println!("{} is eating... {:?}", &self.name, std::thread::current().id() );
        let delay = thread_rng().gen_range(20..50);
        time::sleep(
            time::Duration::from_millis(delay)
        ).await;
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras"];

#[tokio::main]
async fn main() {    
    // Create forks
    let forks = PHILOSOPHERS.iter()
        .fold(vec![], |mut forks, _| {
            forks.push(Arc::new(Mutex::new(Fork)));
            forks
        });
        
    let f = async {
        let (tx,mut rx) = mpsc::channel::<String>(100);
 
        PHILOSOPHERS.iter().enumerate()
            // Create philosophers
            .map(|(i,philosopher)| {
                Philosopher {
                    name: philosopher.to_string(),
                    left_fork: forks[i].clone(),
                    right_fork: forks[ if i == forks.len()-1 { 0 } else { i+1 } ].clone(),
                    thoughts: tx.clone()
                }
            })
            // Make them think and eat
            .all(|ph| {
                tokio::task::spawn_local(async move {
                    for _ in 0..10 {
                        ph.eat().await;
                        ph.think().await;
                    }
                });
                true  
            });
        
        drop(tx);
        // Output their thoughts
        while let Some(thought) = rx.recv().await {
            println!("{}",thought);
        }
    };

    let ls = tokio::task::LocalSet::new();
    ls.run_until(f).await;

}

#[tokio::test]
async fn test_own_thread() {    
    
    // Create forks
    let forks = PHILOSOPHERS.iter()
    .fold(vec![], |mut forks, _| {
        forks.push(Arc::new(Mutex::new(Fork)));
        forks
    });
    
    let (tx,mut rx) = mpsc::channel::<String>(100);
    PHILOSOPHERS.iter().enumerate()
        // Create philosophers
        .map(|(i,philosopher)| {
            Philosopher {
                name: philosopher.to_string(),
                left_fork: forks[i].clone(),
                right_fork: forks[ if i == forks.len()-1 { 0 } else { i+1 } ].clone(),
                thoughts: tx.clone()
            }
        })
        // Make them think and eat
        .all(|ph| {
            !tokio::spawn(async move {
                for _ in 0..10 {
                    ph.eat().await;
                    ph.think().await;
                }
            }).is_finished()
        });
    
    drop(tx);
    // Output their thoughts
    while let Some(thought) = rx.recv().await {
        println!("{}",thought);
    }

}