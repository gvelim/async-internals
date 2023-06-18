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
        time::sleep(time::Duration::from_millis(5)).await;
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras"];

#[tokio::main]
async fn main() {
    let (tx,mut rx) = mpsc::channel::<String>(100);
    
    // Create forks
    let mut forks = vec![];
    for _ in PHILOSOPHERS {
        forks.push(Arc::new(Mutex::new(Fork)))
    }
    for (i,philosopher) in PHILOSOPHERS.iter().enumerate() {
        let ph = Philosopher {
            name: philosopher.to_string(),
            left_fork: if i == 0 { forks.last().unwrap().clone() } else { forks[i].clone() },
            right_fork: forks[i+1].clone(),
            thoughts: tx.clone()
        };

        // Make them think and eat
        let hnd = tokio::task::spawn(async move {
            loop {
                ph.eat().await;
                ph.think().await;
            }
        });
    }

    async move {
        // Create philosophers
        drop(tx);

        // Output their thoughts
        while let Some(thought) = rx.recv().await {
            println!("{}",thought);
        }
    }.await;
}