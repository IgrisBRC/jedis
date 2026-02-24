use std::sync::mpsc::Sender;

use mio::Token;

use crate::{
    temple::Temple,
    wish::{
        Command, Sacrilege, Response, Sin,
        grant::{Decree, Gift},
    },
};

pub fn hget(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() != 3 {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::GET)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    temple.hget(tx, terms[1].clone(), terms[2].clone(), token);

    Ok(())
}
