use mio::Token;

use crate::{
    temple::Temple,
    wish::{grant::{Decree, Gift}, Command, ErrorType, Response, Sin},
};
use std::sync::mpsc::Sender;

pub fn del(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::DEL)),
        }));

        return Ok(());
    }

    temple.del(terms[1..].to_vec(), tx, token);

    Ok(())
}
