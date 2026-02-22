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
            tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Info(InfoType::Ok),
            }));

            return Ok(());
        }
        "CONFIG" => {
            tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Info(InfoType::Ok),
            }));

            return Ok(());
        }
        _ => {
            tx.send(Decree::Deliver(Gift {
                token: token,
                response: Response::Error(ErrorType::UnknownCommand),
            }));

            return Ok(());
        }
    }

    Ok(())
}
