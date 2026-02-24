use crate::{
    temple::Temple,
    wish::{
        Command, Response, Sacrilege, Sin,
        grant::{Decree, Gift},
    },
};
use mio::Token;
use std::sync::mpsc::Sender;

pub fn hmget(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    let terms_len = terms.len();

    if terms_len < 3 {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::HMGET)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        };

        return Ok(());
    }

    let key = terms[1].clone();

    let mut fields = Vec::new();

    let mut idx = 2;

    while idx < terms_len {
        fields.push(terms[idx].clone());

        idx += 1;
    }

    temple.hmget(tx, key, fields, token);

    Ok(())
}
