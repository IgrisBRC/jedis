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
