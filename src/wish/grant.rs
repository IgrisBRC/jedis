
use mio::Token;

use crate::{
    temple::Temple,
    wish::{InfoType, Response, Sacrilege, Sin},
};

use std::sync::mpsc::Sender;

mod append;
mod decr;
mod del;
mod exists;
mod get;
mod hdel;
mod hexists;
mod hget;
mod hlen;
mod hmget;
mod hset;
mod incr;
mod lindex;
mod llen;
mod lpop;
mod lpush;
mod lrange;
mod lset;
mod ping;
mod rpop;
mod rpush;
mod set;
mod strlen;
mod lrem;
mod expire;

pub struct Gift {
    pub token: mio::Token,
    pub response: Response,
}

pub enum Decree {
    Welcome(Token, mio::net::TcpStream),
    Deliver(Gift),
}

pub fn grant(
    terms: Vec<Vec<u8>>,
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    let cmd = &terms[0];

    if cmd.eq_ignore_ascii_case(b"SET") {
        set::set(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"GET") {
        get::get(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"PING") {
        ping::ping(tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"DEL") {
        del::del(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"EXISTS") {
        exists::exists(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"INCR") {
        incr::incr(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"DECR") {
        decr::decr(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"APPEND") {
        append::append(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HSET") {
        hset::hset(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HGET") {
        hget::hget(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HMGET") {
        hmget::hmget(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"STRLEN") {
        strlen::strlen(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HDEL") {
        hdel::hdel(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HEXISTS") {
        hexists::hexists(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"HLEN") {
        hlen::hlen(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LPUSH") {
        lpush::lpush(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LPOP") {
        lpop::lpop(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"RPUSH") {
        rpush::rpush(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"RPOP") {
        rpop::rpop(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LLEN") {
        llen::llen(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LRANGE") {
        lrange::lrange(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LINDEX") {
        lindex::lindex(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LSET") {
        lset::lset(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"LREM") {
        lrem::lrem(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"EXPIRE") {
        expire::expire(terms, temple, tx, token)?
    } else if cmd.eq_ignore_ascii_case(b"COMMAND") || cmd.eq_ignore_ascii_case(b"CONFIG") {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Info(InfoType::Ok),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        };
        return Ok(());
    } else {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::UnknownCommand),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        };
        return Ok(());
    }

    Ok(())
}


// use mio::Token;
//
// use crate::{
//     temple::Temple,
//     wish::{InfoType, Response, Sacrilege, Sin},
// };
//
// use std::sync::mpsc::Sender;
//
// mod append;
// mod decr;
// mod del;
// mod exists;
// mod get;
// mod hdel;
// mod hexists;
// mod hget;
// mod hlen;
// mod hmget;
// mod hset;
// mod incr;
// mod lindex;
// mod llen;
// mod lpop;
// mod lpush;
// mod lrange;
// mod lset;
// mod ping;
// mod rpop;
// mod rpush;
// mod set;
// mod strlen;
// mod lrem;
// mod expire;
//
// pub struct Gift {
//     pub token: mio::Token,
//     pub response: Response,
// }
//
// pub enum Decree {
//     Welcome(Token, mio::net::TcpStream),
//     Deliver(Gift),
// }
//
// pub fn grant(
//     terms: Vec<Vec<u8>>,
//     temple: &mut Temple,
//     tx: Sender<Decree>,
//     token: Token,
// ) -> Result<(), Sin> {
//     match std::str::from_utf8(&terms[0])
//         .map_err(|_| Sin::Disconnected)?
//         .to_uppercase()
//         .as_str()
//     {
//         "SET" => set::set(terms, temple, tx, token)?,
//         "GET" => get::get(terms, temple, tx, token)?,
//         "PING" => ping::ping(tx, token)?,
//         "DEL" => del::del(terms, temple, tx, token)?,
//         "EXISTS" => exists::exists(terms, temple, tx, token)?,
//         "INCR" => incr::incr(terms, temple, tx, token)?,
//         "DECR" => decr::decr(terms, temple, tx, token)?,
//         "APPEND" => append::append(terms, temple, tx, token)?,
//         "HSET" => hset::hset(terms, temple, tx, token)?,
//         "HGET" => hget::hget(terms, temple, tx, token)?,
//         "HMGET" => hmget::hmget(terms, temple, tx, token)?,
//         "STRLEN" => strlen::strlen(terms, temple, tx, token)?,
//         "HDEL" => hdel::hdel(terms, temple, tx, token)?,
//         "HEXISTS" => hexists::hexists(terms, temple, tx, token)?,
//         "HLEN" => hlen::hlen(terms, temple, tx, token)?,
//         "LPUSH" => lpush::lpush(terms, temple, tx, token)?,
//         "LPOP" => lpop::lpop(terms, temple, tx, token)?,
//         "RPUSH" => rpush::rpush(terms, temple, tx, token)?,
//         "RPOP" => rpop::rpop(terms, temple, tx, token)?,
//         "LLEN" => llen::llen(terms, temple, tx, token)?,
//         "LRANGE" => lrange::lrange(terms, temple, tx, token)?,
//         "LINDEX" => lindex::lindex(terms, temple, tx, token)?,
//         "LSET" => lset::lset(terms, temple, tx, token)?,
//         "LREM" => lrem::lrem(terms, temple, tx, token)?,
//         "EXPIRE" => expire::expire(terms, temple, tx, token)?,
//         "COMMAND" => {
//             if tx
//                 .send(Decree::Deliver(Gift {
//                     token,
//                     response: Response::Info(InfoType::Ok),
//                 }))
//                 .is_err()
//             {
//                 eprintln!("angel panicked");
//             };
//
//             return Ok(());
//         }
//         "CONFIG" => {
//             if tx
//                 .send(Decree::Deliver(Gift {
//                     token,
//                     response: Response::Info(InfoType::Ok),
//                 }))
//                 .is_err()
//             {
//                 eprintln!("angel panicked");
//             };
//
//             return Ok(());
//         }
//         _ => {
//             if tx
//                 .send(Decree::Deliver(Gift {
//                     token,
//                     response: Response::Error(Sacrilege::UnknownCommand),
//                 }))
//                 .is_err()
//             {
//                 eprintln!("angel panicked");
//             };
//
//             return Ok(());
//         }
//     }
//
//     Ok(())
// }
