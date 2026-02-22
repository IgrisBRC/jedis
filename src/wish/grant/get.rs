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
