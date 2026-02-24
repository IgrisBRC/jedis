use std::{io::Write, sync::mpsc::Sender, time::SystemTime};

use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{
        Command, Sacrilege, Response, Sin,
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
            response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::INCR)),
        })).is_err() {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.incr(terms[1].clone(), tx, token);

    Ok(())
}
