==> ./src/wish.rs <==
use crate::{temple::{Temple, Value}, wish::grant::{Decree, Gift}};
use mio::{net::TcpStream, Token};
use std::{io::Read, sync::mpsc::Sender, time::SystemTime};

pub enum Phase {
    Idle,
    AwaitingTermCount,
    GraspingMarker,
    AwaitingBulkStringLength,
    AwaitingBulkString(usize),
}

pub struct Virtue {
    backlog: Vec<u8>,
    terms: Vec<Vec<u8>>,
    expected_terms: usize,
    phase: Phase,
}

impl Virtue {
    fn new() -> Self {
        Self {
            backlog: Vec::with_capacity(2048),
            terms: Vec::new(),
            expected_terms: 0,
            phase: Phase::Idle,
        }
    }
}

pub enum Command {
    SET,
    GET,
    EX,
    INCR,
    DECR,
    APPEND,
    EXISTS,
    DEL,
}

pub enum ErrorType {
    IncorrectNumberOfArguments(Command),
    IncorrectUsage(Command),
    UnknownCommand
}

pub enum InfoType {
    Ok,
    Pong
}

pub enum Response {
    Error(ErrorType),
    Info(InfoType),
    BulkString(Option<(Value, Option<SystemTime>)>),
    Amount(u32),
    Number(Option<i64>),
    Length(usize),
}

pub struct Pilgrim {
    pub stream: TcpStream,
    pub virtue: Option<Virtue>,
    pub tx: Sender<Decree>,
}

#[derive(Debug)]
pub enum Sin {
    Utf8Error,
    ParseError,
    Disconnected,
    Blasphemy,
}

pub mod grant;
mod util;

pub fn wish(pilgrim: &mut Pilgrim, mut temple: Temple, token: Token) -> Result<(), Sin> {
    let virtue = pilgrim.virtue.get_or_insert_with(Virtue::new);

    let mut buffer = [0; 1024];

    loop {
        match pilgrim.stream.read(&mut buffer) {
            Ok(0) => return Err(Sin::Disconnected),
            Ok(bytes_read) => {
                virtue.backlog.extend_from_slice(&buffer[..bytes_read]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => return Err(Sin::Disconnected),
        }
    }

    loop {
        if virtue.backlog.is_empty() {
            break;
        }

        match virtue.phase {
            Phase::Idle => {
                if virtue.backlog[0] == b'*' {
                    virtue.phase = Phase::AwaitingTermCount;
                }

                virtue.backlog.drain(..1);
            }
            Phase::AwaitingTermCount => {
                if let Some(index) = util::find_crlf(&virtue.backlog) {
                    virtue.expected_terms = std::str::from_utf8(&virtue.backlog[..index])
                        .map_err(|_| Sin::Utf8Error)?
                        .parse()
                        .map_err(|_| Sin::ParseError)?;

                    if virtue.expected_terms == 0 {
                        return Err(Sin::Blasphemy);
                    }

                    virtue.phase = Phase::GraspingMarker;
                    virtue.backlog.drain(..index + 2);
                } else {
                    break;
                }
            }
            Phase::GraspingMarker => {
                if virtue.backlog[0] == b'$' {
                    virtue.phase = Phase::AwaitingBulkStringLength;
                } else {
                    return Err(Sin::Blasphemy);
                }

                virtue.backlog.drain(..1);
            }
            Phase::AwaitingBulkStringLength => {
                if let Some(index) = util::find_crlf(&virtue.backlog) {
                    let bulk_string_length = std::str::from_utf8(&virtue.backlog[..index])
                        .map_err(|_| Sin::Utf8Error)?
                        .parse()
                        .map_err(|_| Sin::ParseError)?;

                    virtue.phase = Phase::AwaitingBulkString(bulk_string_length);
                    virtue.backlog.drain(..index + 2);
                } else {
                    break;
                }
            }
            Phase::AwaitingBulkString(characters_remaining) => {
                if virtue.backlog.len() >= characters_remaining + 2 {
                    if virtue.backlog[characters_remaining] != b'\r'
                        || virtue.backlog[characters_remaining + 1] != b'\n'
                    {
                        return Err(Sin::Blasphemy);
                    }

                    let term = &virtue.backlog[..characters_remaining];

                    virtue.terms.push(term.into());

                    virtue.backlog.drain(..characters_remaining + 2);
                    virtue.phase = Phase::GraspingMarker;

                    if virtue.terms.len() == virtue.expected_terms {
                        grant::grant(&virtue.terms, &mut temple, pilgrim.tx.clone(), token)?;

                        virtue.terms.clear();
                        virtue.phase = Phase::Idle;
                    }
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

==> ./src/lib.rs <==


pub mod temple;
pub mod choir;
pub mod wish;

==> ./src/wish/util.rs <==

pub fn find_crlf(buffer: &[u8]) -> Option<usize> {
    buffer.windows(2).position(|w| w == b"\r\n")
}

==> ./src/wish/grant/del.rs <==
use mio::Token;

use crate::{
    temple::Temple,
    wish::{grant::{Decree, Gift}, Command, ErrorType, Response, Sin},
};
use std::sync::mpsc::Sender;

pub fn del(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::DEL)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.del(terms[1..].to_vec(), tx, token);

    Ok(())
}

==> ./src/wish/grant/decr.rs <==
use std::{io::Write, sync::mpsc::Sender};

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{grant::{Decree, Gift}, Command, ErrorType, Response, Sin},
};

pub fn decr(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::DECR)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.decr(terms[1].clone(), tx, token);

    Ok(())
}

==> ./src/wish/grant/set.rs <==
use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{
        Command, ErrorType, Response, Sin,
        grant::{Decree, Gift},
    },
};

use std::{sync::mpsc::Sender, time::SystemTime};

pub fn set(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    let terms_len = terms.len();

    if terms_len < 3 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::SET)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    if terms_len == 3 {
        temple.set(
            terms[1].clone(),
            (Value::String(terms[2].clone()), None),
            tx,
            token,
        );
    } else if terms_len == 5 {
        if let Ok(command) = std::str::from_utf8(&terms[3]) {
            if command.to_uppercase() == "EX" {
                if let Ok(expiry) = std::str::from_utf8(&terms[4]) {
                    if let Ok(expiry) = expiry.parse::<u64>() {
                        temple.set(
                            terms[1].clone(),
                            (
                                Value::String(terms[2].clone()),
                                Some(SystemTime::now() + std::time::Duration::from_secs(expiry)),
                            ),
                            tx,
                            token,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

==> ./src/wish/grant/get.rs <==
use std::{io::Write, sync::mpsc::Sender, time::SystemTime};

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{grant::{Decree, Gift}, Command, ErrorType, Response, Sin},
};

pub fn get(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token:Token
) -> Result<(), Sin> {
    if terms.len() < 2 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::GET)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.get(terms[1].clone(), tx, token);

    Ok(())
}

==> ./src/wish/grant/incr.rs <==
use std::{io::Write, sync::mpsc::Sender, time::SystemTime};

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{
        Command, ErrorType, Response, Sin,
        grant::{Decree, Gift},
    },
};

pub fn incr(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::INCR)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.incr(terms[1].clone(), tx, token);

    Ok(())
}

==> ./src/wish/grant/ping.rs <==
use std::sync::mpsc::Sender;

use mio::Token;

use crate::wish::{grant::{Decree, Gift}, InfoType, Response, Sin};

pub fn ping(tx: Sender<Decree>, token: Token) -> Result<(), Sin> {
    if tx.send(Decree::Deliver(Gift {
        token: token,
        response: Response::Info(InfoType::Pong),
    })).is_err() {
        eprintln!("angel panicked");
    };

    Ok(())
}

==> ./src/wish/grant/exists.rs <==
use mio::Token;

use crate::{
    temple::Temple,
    wish::{
        Command, ErrorType, Response, Sin,
        grant::{Decree, Gift},
    },
};
use std::sync::mpsc::Sender;

pub fn exists(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::EXISTS)),
        })).is_err() {
            eprintln!("angel panicked");
        };
        return Ok(());
    }

    temple.exists(terms[1..].to_vec(), tx, token);

    Ok(())
}

==> ./src/wish/grant/append.rs <==
use std::sync::mpsc::Sender;

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{
        Command, ErrorType, Response, Sin,
        grant::{Decree, Gift},
    },
};

pub fn append(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 3 {
        if tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::APPEND)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.append(terms[1].clone(), Value::String(terms[2].clone()), tx, token);

    Ok(())
}

==> ./src/wish/grant.rs <==
use mio::Token;

use crate::{
    temple::Temple,
    wish::{ErrorType, InfoType, Response, Sin},
};

use std::sync::mpsc::Sender;

mod append;
mod decr;
mod del;
mod exists;
mod get;
mod incr;
mod ping;
mod set;

pub struct Gift {
    pub token: mio::Token,
    pub response: Response,
}

pub enum Decree {
    Welcome(Token, mio::net::TcpStream),
    Deliver(Gift),
}

pub fn grant(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    match std::str::from_utf8(&terms[0])
        .map_err(|_| Sin::Disconnected)?
        .to_uppercase()
        .as_str()
    {
        "SET" => set::set(terms, temple, tx, token)?,
        "GET" => get::get(terms, temple, tx, token)?,
        "PING" => ping::ping(tx, token)?,
        "DEL" => del::del(terms, temple, tx, token)?,
        "EXISTS" => exists::exists(terms, temple, tx, token)?,
        "INCR" => incr::incr(terms, temple, tx, token)?,
        "DECR" => decr::decr(terms, temple, tx, token)?,
        "APPEND" => append::append(terms, temple, tx, token)?,
        "COMMAND" => {
            if tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Info(InfoType::Ok),
            })).is_err() {
                eprintln!("angel panicked");
            };

            return Ok(());
        }
        "CONFIG" => {
            if tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Info(InfoType::Ok),
            })).is_err() {
                eprintln!("angel panicked");
            };

            return Ok(());
        }
        _ => {
            if tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Error(ErrorType::UnknownCommand),
            })).is_err() {
                eprintln!("angel panicked");
            };

            return Ok(());
        }
    }

    Ok(())
}

==> ./src/temple.rs <==
use std::collections::hash_map::Entry;
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::Sender;
use std::{collections::HashMap, time::SystemTime};

use mio::Token;

use crate::wish::Response;
use crate::wish::grant::{Decree, Gift};

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

    pub fn get(&mut self, key: Vec<u8>) -> Option<(Value, Option<SystemTime>)> {
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

    pub fn set(
        &mut self,
        key: Vec<u8>,
        val: (Value, Option<SystemTime>),
    ) -> Option<(Value, Option<SystemTime>)> {
        self.0.insert(key, val)
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

    pub fn incr(&mut self, key: Vec<u8>) -> Option<i64> {
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

                        return Some(existing_value + 1);
                    }
                }

                self.0.insert(key, (Value::String(existing_value), expiry));
                None
            }

            Some((other_value, expiry)) => {
                self.0.insert(key, (other_value, expiry));
                None
            }

            None => {
                let initial = Value::String(b"1".to_vec());
                self.0.insert(key, (initial, None));

                Some(1)
            }
        }
    }

    pub fn decr(&mut self, key: Vec<u8>) -> Option<i64> {
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

                        return Some(existing_value - 1);
                    }
                }

                self.0.insert(key, (Value::String(existing_value), expiry));
                None
            }

            Some((existing_value, expiry)) => {
                self.0.insert(key, (existing_value, expiry));
                None
            }

            None => {
                let initial = Value::String(b"-1".to_vec());
                self.0.insert(key, (initial, None));

                Some(-1)
            }
        }
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
                        Wish::Get { key, token, tx } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::BulkString(soul.get(key)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Set {
                            key,
                            token,
                            value: val,
                            tx,
                        } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::BulkString(soul.set(key, val)),
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
                        Wish::Incr { key, token, tx } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Number(soul.incr(key)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
                        Wish::Decr { key, token, tx } => {
                            if tx
                                .send(Decree::Deliver(Gift {
                                    token,
                                    response: Response::Number(soul.decr(key)),
                                }))
                                .is_err()
                            {
                                eprintln!("angel panicked");
                            }
                        }
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

    pub fn sanctify(&self) -> Self {
        self.clone()
    }
}

==> ./src/egress.rs <==
use jerusalem::{
    temple::Value,
    wish::{InfoType, Response, Sin, grant::Gift},
};
use mio::net::TcpStream;
use std::io::Write;

pub fn egress(stream: &mut TcpStream, gift: Gift) -> Result<(), Sin> {
    let mut response: Vec<u8> = Vec::new();

    match gift.response {
        Response::Info(InfoType::Ok) => {
            response.append(&mut b"+OK\r\n".to_vec());
        }
        Response::Info(InfoType::Pong) => {
            response.append(&mut b"+PONG\r\n".to_vec());
        }
        Response::BulkString(bulk_string) => match bulk_string {
            Some((value, _)) => {
                if let Value::String(mut value) = value {
                    response.append(&mut format!("${}\r\n", value.len()).into_bytes());
                    response.append(&mut value);
                    response.append(&mut "\r\n".as_bytes().to_vec());
                }
            }
            None => {
                response.append(&mut b"$-1\r\n".to_vec());
            }
        },
        Response::Amount(amount) => {
            response.append(&mut format!(":{}\r\n", amount).into_bytes());
        }
        Response::Number(number) => match number {
            Some(number) => {
                let mut number_string = number.to_string().into_bytes();

                response.push(b':');
                response.append(&mut number_string);
                response.append(&mut "\r\n".as_bytes().to_vec());
            }
            None => {
                response.append(&mut
                    b"-ERR I don't know what you might have done to get this one.".to_vec(),
                );
            }
        },
        Response::Length(length) => {
            response.append(&mut format!(":{}\r\n", length).into_bytes());
        }
        Response::Error(_) => {
            response.append(&mut b"-ERR Some error occured, and because I was too impatient to test this I didn't really wanna write out the logic to match my way through to figure out which error has happened here.\r\n".to_vec());
        }
    }

    stream.write_all(&response).map_err(|_| Sin::Disconnected)?;

    Ok(())
}

==> ./src/main.rs <==
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};

use jerusalem::choir::Choir;
use jerusalem::temple::Temple;
use jerusalem::wish::grant::Decree;
use jerusalem::wish::{self, Pilgrim};
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

mod egress;

fn main() {
    let ipv4_addr = Ipv4Addr::new(127, 0, 0, 1);
    let port = 6379;
    let socket_addr_v4 = SocketAddrV4::new(ipv4_addr, port);
    let socket_addr = SocketAddr::V4(socket_addr_v4);

    let mut poll = Poll::new().unwrap();

    let mut listener = TcpListener::bind(socket_addr).unwrap();

    const SERVER: Token = Token(0);

    let mut events = Events::with_capacity(128);

    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)
        .unwrap();

    let mut ingress_map: HashMap<Token, Pilgrim> = HashMap::new();

    let mut pilgrim_counter = 1;

    let ingress_choir = Choir::new(5);

    let temple = Temple::new("IgrisDB");

    let (ingress_tx, ingress_rx) = std::sync::mpsc::channel();
    let (egress_tx, egress_rx) = std::sync::mpsc::channel();

    let (pilgrim_tx, pilgrim_rx) = std::sync::mpsc::channel::<Decree>();

    std::thread::spawn(move || {
        let mut egress_map: HashMap<Token, mio::net::TcpStream> = HashMap::new();

        loop {
            match pilgrim_rx.recv() {
                Ok(Decree::Welcome(token, stream)) => {
                    egress_map.insert(token, stream);
                }
                Ok(Decree::Deliver(gift)) => {
                    if let Some(stream) = egress_map.get_mut(&gift.token) {
                        let token = gift.token;

                        if let Err(_) = egress::egress(stream, gift) {
                            if egress_tx.send(token).is_err() {
                                eprintln!("angel panicked");
                            };
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    loop {
        while let Ok((token, pilgrim)) = ingress_rx.try_recv() {
            ingress_map.insert(token, pilgrim);
        }

        while let Ok(token) = egress_rx.try_recv() {
            if let Some(mut pilgrim) = ingress_map.remove(&token) {
                if poll.registry().deregister(&mut pilgrim.stream).is_err() {
                    eprintln!("deregister() failed")
                }
            }
        }

        if poll
            .poll(&mut events, Some(std::time::Duration::from_millis(10)))
            .is_err()
        {
            eprintln!("poll() gone wrong");
        }

        for event in &events {
            let token = event.token();
            match token {
                SERVER => loop {
                    match listener.accept() {
                        Ok((mut stream, _address)) => {
                            let pilgrim_token = Token(pilgrim_counter);

                            if poll
                                .registry()
                                .register(
                                    &mut stream,
                                    pilgrim_token,
                                    Interest::READABLE | Interest::WRITABLE,
                                )
                                .is_err()
                            {
                                eprintln!("register() gone wrong");
                            }

                            let std_stream: TcpStream = stream.into();
                            let std_stream_clone: TcpStream =
                                std_stream.try_clone().expect("Failed to clone socket");

                            let ingress_mio = mio::net::TcpStream::from_std(std_stream);
                            let egress_mio = mio::net::TcpStream::from_std(std_stream_clone);

                            pilgrim_counter += 1;

                            ingress_map.insert(
                                pilgrim_token,
                                Pilgrim {
                                    stream: ingress_mio,
                                    virtue: None,
                                    tx: pilgrim_tx.clone(),
                                },
                            );

                            pilgrim_tx
                                .send(Decree::Welcome(pilgrim_token, egress_mio))
                                .unwrap();
                        }
                        Err(err) => {
                            if err.kind() == ErrorKind::WouldBlock {
                                break;
                            }
                        }
                    }
                },

                Token(token_number) => {
                    if let Some(mut pilgrim) = ingress_map.remove(&Token(token_number)) {
                        let sanctum = temple.sanctify();
                        let token_number = token_number;
                        let tx = ingress_tx.clone();

                        ingress_choir.sing(move || {
                            match wish::wish(&mut pilgrim, sanctum, Token(token_number)) {
                                Ok(_) => {
                                    if tx.send((mio::Token(token_number), pilgrim)).is_err() {
                                        eprintln!("angel panicked");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}

==> ./src/choir.rs <==
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
