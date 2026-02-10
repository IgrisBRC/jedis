use std::collections::hash_map::Entry;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, time::SystemTime};

pub struct Soul(HashMap<String, (Vec<u8>, Option<SystemTime>)>);

impl Soul {
    pub fn new() -> Self {
        Soul(HashMap::new())
    }

    pub fn get(&mut self, key: String) -> Option<(Vec<u8>, Option<SystemTime>)> {
        match self.0.entry(key) {
            Entry::Occupied(occupied) => {
                let (data, expiry) = occupied.get();

                if let Some(time) = expiry {
                    if *time < SystemTime::now() {
                        occupied.remove();
                        return None;
                    }
                }

                Some((data.clone(), *expiry))
            }
            Entry::Vacant(_) => None,
        }
    }

    pub fn insert(
        &mut self,
        key: String,
        val: (Vec<u8>, Option<SystemTime>),
    ) -> Option<(Vec<u8>, Option<SystemTime>)> {
        self.0.insert(key, val)
    }

    pub fn remove(&mut self, key: String) -> Option<(Vec<u8>, Option<SystemTime>)> {
        self.0.remove(&key)
    }
}

pub enum Wish {
    Get {
        key: String,
        tx: Sender<Option<(Vec<u8>, Option<SystemTime>)>>,
    },
    Insert {
        key: String,
        val: (Vec<u8>, Option<SystemTime>),
        tx: Sender<Option<(Vec<u8>, Option<SystemTime>)>>,
    },
    Remove {
        key: String,
        tx: Sender<Option<(Vec<u8>, Option<SystemTime>)>>,
    },
    Incr {
        key: String,
        tx: Sender<Option<i64>>,
    },
}

#[derive(Clone)]
pub struct Temple {
    _name: String,
    tx: Sender<Wish>,
}

impl Temple {
    pub fn new(name: String) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut soul = Soul::new();

            while let Ok(wish) = rx.recv() {
                match wish {
                    Wish::Get { key, tx } => {
                        let _ = tx.send(soul.get(key));
                    }
                    Wish::Insert { key, val, tx } => {
                        let _ = tx.send(soul.insert(key, val));
                    }
                    Wish::Remove { key, tx } => {
                        let _ = tx.send(soul.remove(key));
                    }
                    Wish::Incr { key, tx } => {
                        if let Some((value_bytes, expiry)) = soul.get(key.clone()) {
                            if let Ok(value) = String::from_utf8_lossy(&value_bytes).parse::<i64>()
                            {
                                let incremented_value = value + 1;
                                soul.insert(
                                    key,
                                    (incremented_value.to_string().into_bytes(), expiry),
                                );

                                let _ = tx.send(Some(value));
                            } else {
                                let _ = tx.send(None);
                            }
                        } else {
                            soul.insert(key, (1.to_string().into_bytes(), None));
                            let _ = tx.send(Some(1));
                        }
                    }
                }
            }
        });

        Temple { _name: name, tx }
    }

    pub fn get(&self, key: String) -> Option<(Vec<u8>, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        let _ = self.tx.send(Wish::Get { key, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn insert(
        &self,
        key: String,
        val: (Vec<u8>, Option<SystemTime>),
    ) -> Option<(Vec<u8>, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        let _ = self.tx.send(Wish::Insert { key, val, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn remove(&self, key: String) -> Option<(Vec<u8>, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        let _ = self.tx.send(Wish::Remove { key, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn incr(&self, key: String) -> Option<i64> {
        let (tx, rx) = std::sync::mpsc::channel();

        let _ = self.tx.send(Wish::Incr { key, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn sanctify(&self) -> Self {
        self.clone()
    }
}
