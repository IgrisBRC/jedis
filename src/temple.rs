use std::collections::hash_map::Entry;
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, time::SystemTime};

#[derive(Clone)]
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Hash(HashMap<Vec<u8>, Vec<u8>>),
    Set(HashSet<Vec<u8>>),
}

pub struct Soul(HashMap<String, (Value, Option<SystemTime>)>);

impl Soul {
    pub fn new() -> Self {
        Soul(HashMap::new())
    }

    pub fn get(&mut self, key: String) -> Option<(Value, Option<SystemTime>)> {
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
        val: (Value, Option<SystemTime>),
    ) -> Option<(Value, Option<SystemTime>)> {
        self.0.insert(key, val)
    }

    pub fn remove(&mut self, key: String) -> Option<(Value, Option<SystemTime>)> {
        self.0.remove(&key)
    }
}

pub enum Wish {
    Get {
        key: String,
        tx: Sender<Option<(Value, Option<SystemTime>)>>,
    },
    Insert {
        key: String,
        val: (Value, Option<SystemTime>),
        tx: Sender<Option<(Value, Option<SystemTime>)>>,
    },
    Remove {
        key: String,
        tx: Sender<Option<(Value, Option<SystemTime>)>>,
    },
}

#[derive(Clone)]
pub struct Temple {
    name: String,
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
                        tx.send(soul.get(key));
                    }
                    Wish::Insert { key, val, tx } => {
                        tx.send(soul.insert(key, val));
                    }
                    Wish::Remove { key, tx } => {
                        tx.send(soul.remove(key));
                    }
                }
            }
        });

        Temple { name, tx }
    }

    pub fn get(&self, key: String) -> Option<(Value, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        self.tx.send(Wish::Get { key, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn insert(
        &self,
        key: String,
        value: (Value, Option<SystemTime>),
    ) -> Option<(Value, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        self.tx.send(Wish::Insert { key, val: value, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn remove(&self, key: String) -> Option<(Value, Option<SystemTime>)> {
        let (tx, rx) = std::sync::mpsc::channel();

        self.tx.send(Wish::Remove { key, tx });

        rx.recv().unwrap_or(None)
    }

    pub fn sanctify(&self) -> Self {
        self.clone()
    }
}
