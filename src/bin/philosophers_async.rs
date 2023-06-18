use std::sync::Arc;
use tokio::task::spawn_local;
use tokio::{time, spawn};
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
        let _rf = self.right_fork.lock().await;
        println!("{} is eating...", &self.name);
        time::sleep(time::Duration::from_millis(50)).await;
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras"];

#[tokio::main]
async fn main() {
    let (tx,mut rx) = mpsc::channel::<String>(100);
    
    // Create forks
    let forks = PHILOSOPHERS.iter()
        .fold(vec![], |mut forks, _| {
            forks.push(Arc::new(Mutex::new(Fork)));
            forks
        });
        
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
            !spawn(async move {
                for _ in 0..10 {
                    ph.eat().await;
                    ph.think().await;
                }
            }).is_finished()  
        });
    
    drop(tx);
    async move {
        // Output their thoughts
        while let Some(thought) = rx.recv().await {
            println!("{}",thought);
        }
    }.await;
}