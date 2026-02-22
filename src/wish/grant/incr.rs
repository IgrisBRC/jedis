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
        tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::INCR)),
        }));

        return Ok(());
    }

    temple.incr(terms[1].clone(), tx, token);

    Ok(())
}
