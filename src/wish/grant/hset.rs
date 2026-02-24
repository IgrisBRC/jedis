use mio::Token;

use crate::{
    temple::Temple,
    wish::{
        Command, Sacrilege, Response, Sin,
        grant::{Decree, Gift},
    },
};
use std::sync::mpsc::Sender;

pub fn hset(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    let terms_len = terms.len();

    if terms_len < 4 || terms_len % 2 != 0 {
        if tx
            .send(Decree::Deliver(Gift {
                token,
                response: Response::Error(Sacrilege::IncorrectNumberOfArguments(Command::HSET)),
            }))
            .is_err()
        {
            eprintln!("angel panicked");
        };
        return Ok(());
    }

    let key = terms[1].clone();

    let mut field_value_pairs = Vec::new();

    let mut idx = 2;

    while idx < terms_len - 1 {
        let field_value_pair = (terms[idx].clone(), terms[idx + 1].clone());
        field_value_pairs.push(field_value_pair);

        idx += 2;
    }

    temple.hset(key, field_value_pairs, tx, token);

    Ok(())
}
