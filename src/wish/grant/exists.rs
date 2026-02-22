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
        tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::EXISTS)),
        }));
        return Ok(());
    }

    temple.exists(terms[1..].to_vec(), tx, token);

    Ok(())
}
