use std::collections::hash_map::Entry;
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, time::SystemTime};

use mio::Token;

use crate::wish::grant::{Decree, Gift};
use crate::wish::{Command, InfoType, Response, Sacrilege};

#[derive(Clone)]
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Hash(HashMap<Vec<u8>, Vec<u8>>),
    Set(HashSet<Vec<u8>>),
}

pub struct Soul(HashMap<Vec<u8>, (Value, Option<SystemTime>)>);

impl Soul {
    pub fn new() -> Self {
        Soul(HashMap::new())
    }

    pub fn get(&mut self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Sacrilege> {
        match self.0.entry(key) {
            Entry::Occupied(occupied) => {
                let (value, expiry) = occupied.get();

                if let Some(time) = expiry {
                    if *time < SystemTime::now() {
                        occupied.remove();
                        return Ok(None);
                    }
                }

                if let Value::String(value) = value {
                    return Ok(Some(value.to_vec()));
                } else {
                    return Err(Sacrilege::IncorrectUsage(Command::GET));
                }
            }
            Entry::Vacant(_) => Ok(None),
        }
    }

    pub fn set(&mut self, key: Vec<u8>, val: (Value, Option<SystemTime>)) {
        self.0.insert(key, val);
    }

    pub fn append(&mut self, key: Vec<u8>, incoming_value: Value) -> usize {
        let Value::String(mut incoming_value) = incoming_value else {
            return 0;
        };

        let entry = self.0.remove(&key);

        match entry {
            Some((Value::String(mut existing_value), Some(time))) if time >= SystemTime::now() => {
                existing_value.append(&mut incoming_value);

                let length = existing_value.len();

                self.0
                    .insert(key, (Value::String(existing_value), Some(time)));

                length
            }
            Some((Value::String(mut existing_value), None)) => {
                existing_value.append(&mut incoming_value);

                let length = existing_value.len();

                self.0.insert(key, (Value::String(existing_value), None));

                length
            }
            Some((_, _)) => 0,
            None => {
                let length = incoming_value.len();

                self.0.insert(key, (Value::String(incoming_value), None));

                length
            }
        }
    }

    pub fn incr(&mut self, key: Vec<u8>) -> Result<i64, Sacrilege> {
        let entry = self.0.remove(&key);

        match entry {
            Some((Value::String(existing_value), expiry)) => {
                if let Ok(existing_value) = std::str::from_utf8(&existing_value) {
                    if let Ok(existing_value) = existing_value.parse::<i64>() {
                        self.0.insert(
                            key,
                            (
                                Value::String((existing_value + 1).to_string().into_bytes()),
                                expiry,
                            ),
                        );

                        return Ok(existing_value + 1);
                    }
                }

                self.0.insert(key, (Value::String(existing_value), expiry));
                Err(Sacrilege::IncorrectUsage(Command::INCR))
            }

            Some((other_value, expiry)) => {
                self.0.insert(key, (other_value, expiry));
                Err(Sacrilege::IncorrectUsage(Command::INCR))
            }

            None => {
                let initial = Value::String(b"1".to_vec());
                self.0.insert(key, (initial, None));

                Ok(1)
            }
        }
    }

    pub fn decr(&mut self, key: Vec<u8>) -> Result<i64, Sacrilege> {
        let entry = self.0.remove(&key);

        match entry {
            Some((Value::String(existing_value), expiry)) => {
                if let Ok(existing_value) = std::str::from_utf8(&existing_value) {
                    if let Ok(existing_value) = existing_value.parse::<i64>() {
                        self.0.insert(
                            key,
                            (
                                Value::String((existing_value - 1).to_string().into_bytes()),
                                expiry,
                            ),
                        );

                        return Ok(existing_value - 1);
                    }
                }

                self.0.insert(key, (Value::String(existing_value), expiry));
                Err(Sacrilege::IncorrectUsage(Command::DECR))
            }

            Some((existing_value, expiry)) => {
                self.0.insert(key, (existing_value, expiry));
                Err(Sacrilege::IncorrectUsage(Command::DECR))
            }

            None => {
                let initial = Value::String(b"-1".to_vec());
                self.0.insert(key, (initial, None));

                Ok(-1)
            }
        }
    }

    pub fn del(&mut self, keys: Vec<Vec<u8>>) -> u32 {
        let mut number_of_entries_deleted = 0;

        for key in keys {
            if self.0.remove(&key).is_some() {
                number_of_entries_deleted += 1;
            }
        }

        number_of_entries_deleted
    }

    pub fn exists(&self, keys: Vec<Vec<u8>>) -> u32 {
        let mut number_of_entries_that_exist = 0;

        for key in keys {
            if self.0.get(&key).is_some() {
                number_of_entries_that_exist += 1;
            }
        }

        number_of_entries_that_exist
    }

    pub fn hset(
        &mut self,
        key: Vec<u8>,
        field_value_pairs: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<u32, Sacrilege> {
        match self.0.get_mut(&key) {
            Some((Value::Hash(map), _)) => {
                let mut new_values_added = 0;

                for field_value_pair in field_value_pairs {
                    let (field, value) = field_value_pair;

                    if map.insert(field, value).is_none() {
                        new_values_added += 1;
                    }
                }

                Ok(new_values_added)
            }
            Some(_) => Err(Sacrilege::IncorrectUsage(Command::HSET)),
            None => {
                let mut map = HashMap::new();
                let mut new_values_added = 0;

                for field_value_pair in field_value_pairs {
                    let (field, value) = field_value_pair;

                    map.insert(field, value);
                    new_values_added += 1;
                }

                self.0.insert(key, (Value::Hash(map), None));

                Ok(new_values_added)
            }
        }
    }

    pub fn hget(&mut self, key: Vec<u8>, field: Vec<u8>) -> Result<Option<Vec<u8>>, Sacrilege> {
        match self.0.get(&key) {
            Some((Value::Hash(map), _)) => Ok(map.get(&field).cloned()),
            Some(_) => Err(Sacrilege::IncorrectUsage(Command::HGET)),
            None => Ok(None),
        }
    }

    pub fn hmget(
        &mut self,
        key: Vec<u8>,
        fields: Vec<Vec<u8>>,
    ) -> Result<Option<Vec<Option<Vec<u8>>>>, Sacrilege> {
        let mut values = Vec::new();

        match self.0.get(&key) {
            Some((Value::Hash(map), _)) => {
                for field in fields {
                    values.push(map.get(&field).cloned());
                }
            }
            Some(_) => return Err(Sacrilege::IncorrectUsage(Command::HGET)),
            None => return Ok(None),
        }

        Ok(Some(values))
    }
}

pub enum Wish {
    Get {
        key: Vec<u8>,
        token: Token,
        tx: Sender<Decree>,
    },
    Set {
        key: Vec<u8>,
        token: Token,
        value: (Value, Option<SystemTime>),
        tx: Sender<Decree>,
    },
    Del {
        keys: Vec<Vec<u8>>,
        token: Token,
        tx: Sender<Decree>,
    },
    Append {
        key: Vec<u8>,
        token: Token,
        value: Value,
        tx: Sender<Decree>,
    },
    Incr {
        key: Vec<u8>,
        token: Token,
        tx: Sender<Decree>,
    },
    Decr {
        key: Vec<u8>,
        token: Token,
        tx: Sender<Decree>,
    },
    Exists {
        keys: Vec<Vec<u8>>,
        token: Token,
        tx: Sender<Decree>,
    },
    Hset {
        key: Vec<u8>,
        token: Token,
        field_value_pairs: Vec<(Vec<u8>, Vec<u8>)>,
        tx: Sender<Decree>,
    },
    Hget {
        key: Vec<u8>,
        token: Token,
        field: Vec<u8>,
        tx: Sender<Decree>,
    },
    Hmget {
        key: Vec<u8>,
        token: Token,
        fields: Vec<Vec<u8>>,
        tx: Sender<Decree>,
    },
}

#[derive(Clone)]
pub struct Temple<'a> {
    name: &'a str,
    tx: Sender<Wish>,
}

impl<'a> Temple<'a> {
    pub fn new(name: &'a str) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut soul = Soul::new();

            loop {
                match rx.recv() {
                    Ok(wish) => match wish {
                        Wish::Get { key, token, tx } => match soul.get(key) {
                            Ok(bulk_string) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::BulkString(bulk_string),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                        },
                        Wish::Set {
                            key,
                            token,
                            value: val,
                            tx,
                        } => {
                            soul.set(key, val);

                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Info(InfoType::Ok),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Del { keys, token, tx } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Amount(soul.del(keys)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Append {
                            key,
                            token,
                            value: val,
                            tx,
                        } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Length(soul.append(key, val)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Incr { key, token, tx } => match soul.incr(key) {
                            Ok(number) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Number(number),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                        },
                        Wish::Decr { key, token, tx } => match soul.decr(key) {
                            Ok(number) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Number(number),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                        },
                        Wish::Exists { keys, token, tx } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Amount(soul.exists(keys)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Hset {
                            key,
                            token,
                            field_value_pairs,
                            tx,
                        } => match soul.hset(key, field_value_pairs) {
                            Ok(new_values_added) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token: token,
                                        response: Response::Amount(new_values_added),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                };
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token: token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                };
                            }
                        },
                        Wish::Hget {
                            key,
                            token,
                            field,
                            tx,
                        } => match soul.hget(key, field) {
                            Ok(bulk_string) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::BulkString(bulk_string),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                        },
                        Wish::Hmget {
                            key,
                            token,
                            fields,
                            tx,
                        } => match soul.hmget(key, fields) {
                            Ok(bulk_string_array) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::BulkStringArray(bulk_string_array),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                            Err(sacrilege) => {
                                if tx
                                    .send(Decree::Deliver(Gift {
                                        token,
                                        response: Response::Error(sacrilege),
                                    }))
                                    .is_err()
                                {
                                    eprintln!("angel panicked");
                                }
                            }
                        },
                    },
                    Err(e) => {
                        eprintln!("GodThread: {}", e);
                        break;
                    }
                }
            }
        });

        Temple { name: &name, tx }
    }

    pub fn get(&self, key: Vec<u8>, tx: Sender<Decree>, token: Token) {
        if self.tx.send(Wish::Get { key, token, tx }).is_err() {
            eprintln!("angel panicked");
        }
    }

    pub fn set(
        &self,
        key: Vec<u8>,
        value: (Value, Option<SystemTime>),
        tx: Sender<Decree>,
        token: Token,
    ) {
        if self
            .tx
            .send(Wish::Set {
                key,
                token,
                value,
                tx,
            })
            .is_err()
        {
            eprintln!("angel panicked");
        }
    }

    pub fn del(&self, keys: Vec<Vec<u8>>, tx: Sender<Decree>, token: Token) {
        if self.tx.send(Wish::Del { keys, token, tx }).is_err() {
            eprintln!("angel panicked");
        }
    }

    pub fn exists(&self, keys: Vec<Vec<u8>>, tx: Sender<Decree>, token: Token) {
        if self.tx.send(Wish::Exists { keys, token, tx }).is_err() {
            eprintln!("angel panicked");
        }
    }

    pub fn append(&self, key: Vec<u8>, value: Value, tx: Sender<Decree>, token: Token) {
        if self
            .tx
            .send(Wish::Append {
                key,
                token,
                value,
                tx,
            })
            .is_err()
        {
            eprintln!("angel panicked");
        }
    }

    pub fn incr(&self, key: Vec<u8>, tx: Sender<Decree>, token: Token) {
        if self.tx.send(Wish::Incr { key, token, tx }).is_err() {
            eprintln!("angel panicked");
        }
    }

    pub fn decr(&self, key: Vec<u8>, tx: Sender<Decree>, token: Token) {
        if self.tx.send(Wish::Decr { key, token, tx }).is_err() {
            eprintln!("angel panicked");
        }
    }

    pub fn hset(
        &self,
        key: Vec<u8>,
        field_value_pairs: Vec<(Vec<u8>, Vec<u8>)>,
        tx: Sender<Decree>,
        token: Token,
    ) {
        if self
            .tx
            .send(Wish::Hset {
                key,
                token,
                field_value_pairs,
                tx,
            })
            .is_err()
        {
            eprintln!("angel panicked");
        }
    }

    pub fn hget(&self, tx: Sender<Decree>, key: Vec<u8>, field: Vec<u8>, token: Token) {
        if self
            .tx
            .send(Wish::Hget {
                key,
                token,
                field,
                tx,
            })
            .is_err()
        {
            eprintln!("angel panicked");
        }
    }

    pub fn hmget(&self, tx: Sender<Decree>, key: Vec<u8>, fields: Vec<Vec<u8>>, token: Token) {
        if self
            .tx
            .send(Wish::Hmget {
                key,
                token,
                fields,
                tx,
            })
            .is_err()
        {
            eprintln!("angel panicked");
        }
    }

    pub fn sanctify(&self) -> Self {
        self.clone()
    }
}
