use std::sync::{Arc, Mutex, mpsc::Receiver, mpsc::Sender};

type Song = Box<dyn FnOnce() + Send + 'static>;

struct Angel {
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Angel {
    fn new(rx: Arc<Mutex<Receiver<Song>>>) -> Self {
        Angel {
            thread: Some(std::thread::spawn(move || {
                loop {
                    let song = {
                        let Ok(guard) = rx.lock() else {
                            break;
                        };
                        let Ok(song) = guard.recv() else {
                            break;
                        };
                        song
                    };
                    song();
                }
            })),
        }
    }
}

pub struct Choir {
    angels: Vec<Angel>,
    tx: Option<Sender<Song>>,
}

impl Choir {
    pub fn new(capacity: usize) -> Self {
        let mut angels = Vec::with_capacity(capacity);
        let (tx, rx) = std::sync::mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        for _ in 0..capacity {
            angels.push(Angel::new(Arc::clone(&rx)));
        }

        Choir {
            angels,
            tx: Some(tx),
        }
    }

    pub fn sing<F>(&self, song: F)
    where 
    F: FnOnce() + Send + 'static
    {
        if let Some(tx) = &self.tx {
            tx.send(Box::new(song)).unwrap();
        }
    }
}

impl Drop for Choir {
    fn drop(&mut self) {
        drop(self.tx.take());

        for angel in &mut self.angels {
            angel.thread.take().unwrap().join().unwrap();
        }
    }
}
