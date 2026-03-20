use std::sync::mpsc::Sender;

use mio::Token;

use crate::{
    temple::Temple,
    wish::{
        Command, Response, Sacrilege,
        grant::{Decree, Gift},
    },
};

pub fn config(terms: Vec<Vec<u8>>, temple: &mut Temple, tx: Sender<Decree>, token: Token) {
    if terms.len() < 3 {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::CONFIG)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        }

        return;
    }

    let mut terms_iter = terms.into_iter();
    terms_iter.next();

    let Some(command) = terms_iter.next() else {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::CONFIG)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        }

        return;
    };

    if !command.eq_ignore_ascii_case(b"GET") {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectUsage(Command::CONFIG)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        }

        return;
    }

    temple.config_get(tx, token, terms_iter.collect());
}
