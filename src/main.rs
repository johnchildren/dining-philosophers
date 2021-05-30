use std::{
    sync::{Arc, Mutex, TryLockError},
    thread,
    time::Duration,
};

use rand::{thread_rng, Rng};

struct Philosopher {
    name: String,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
}

#[derive(Debug)]
struct ForkInUse;

impl Philosopher {
    fn new(name: String, left_fork: Arc<Mutex<Fork>>, right_fork: Arc<Mutex<Fork>>) -> Self {
        Self {
            name,
            left_fork,
            right_fork,
        }
    }

    fn think(&self) {
        std::thread::sleep(Duration::from_nanos(thread_rng().gen::<u64>() % 10))
    }

    fn with_left_fork<F, R>(&self, f: F) -> Result<R, ForkInUse>
    where
        F: FnOnce(&Fork) -> Result<R, ForkInUse>,
    {
        match self.left_fork.try_lock() {
            Err(TryLockError::Poisoned(_)) => {
                panic!("someone poisoned the fork!")
            }
            Err(TryLockError::WouldBlock) => Err(ForkInUse),
            Ok(fork) => {
                self.exclaim("picked up the left fork");
                let res = f(&fork);
                self.exclaim("dropped the right fork");
                res
            }
        }
    }

    fn with_right_fork<F, R>(&self, f: F) -> Result<R, ForkInUse>
    where
        F: FnOnce(&Fork) -> Result<R, ForkInUse>,
    {
        match self.right_fork.try_lock() {
            Err(TryLockError::Poisoned(_)) => {
                panic!("someone poisoned the fork!")
            }
            Err(TryLockError::WouldBlock) => Err(ForkInUse),
            Ok(fork) => {
                self.exclaim("picked up the right fork");
                let res = f(&fork);
                self.exclaim("dropped the right fork");
                res
            }
        }
    }

    fn with_left_fork_blocking<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Fork) -> R,
    {
        match self.left_fork.lock() {
            Err(_) => {
                panic!("someone poisoned the fork!")
            }
            Ok(fork) => {
                self.exclaim("picked up the right fork");
                let res = f(&fork);
                self.exclaim("dropped the right fork");
                res
            }
        }
    }

    fn with_right_fork_blocking<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Fork) -> R,
    {
        match self.right_fork.lock() {
            Err(_) => {
                panic!("someone poisoned the fork!")
            }
            Ok(fork) => {
                self.exclaim("picked up the right fork");
                let res = f(&fork);
                self.exclaim("dropped the right fork");
                res
            }
        }
    }

    /// In order to eat, I need a reference to each fork.
    fn eat(&self, _left_fork: &Fork, _right_fork: &Fork) {
        std::thread::sleep(Duration::from_nanos(100))
    }

    fn exclaim(&self, msg: &str) {
        println!("{}: {}", self.name, msg);
    }

    fn dine(&self) {
        self.exclaim("I have sat down");
        for _ in 0..1000000 {
            if self
                .with_left_fork(|left_fork| {
                    self.with_right_fork(|right_fork| {
                        self.exclaim("Time to chow down!");
                        self.eat(left_fork, right_fork);
                        Ok(())
                    })
                })
                .is_err()
            {
                self.exclaim("I can't eat, so I'll think instead");
                self.think();
            }
        }
    }

    fn dine_blocking(&self) {
        self.exclaim("I have sat down");
        for _ in 0..1000000 {
            self.with_left_fork_blocking(|left_fork| {
                self.with_right_fork_blocking(|right_fork| {
                    self.exclaim("Time to chow down!");
                    self.eat(left_fork, right_fork);
                })
            });
        }
    }
}

struct Fork {}

static NAMES: &[&str] = &[
    "Hobbes",
    "Locke",
    "Plato",
    "Socrates",
    "Hume",
    "Russell",
    "Kant",
    "Hegel",
    "Marx",
    "Aristotle",
];

fn main() {
    let total_philosophers = 5;
    let total_forks = total_philosophers;

    let mut forks = Vec::new();
    for _ in 0..total_forks {
        forks.push(Arc::new(Mutex::new(Fork {})))
    }

    let mut handles = Vec::new();
    for i in 0..total_philosophers {
        let left_fork = if i == 0 {
            forks[total_forks - 1].clone()
        } else {
            forks[i - 1].clone()
        };

        let right_fork = forks[i].clone();
        handles.push(thread::spawn(move || {
            let philosopher = Philosopher::new(NAMES[i].to_string(), left_fork, right_fork);

            philosopher.dine();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
