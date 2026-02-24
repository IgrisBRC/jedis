use std::{io::Write, sync::mpsc::Sender};

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{grant::{Decree, Gift}, Command, Sacrilege, Response, Sin},
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
            response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::DECR)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.decr(terms[1].clone(), tx, token);

    Ok(())
}
