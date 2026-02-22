use mio::Token;

use crate::{
    temple::{Temple, Value},
    wish::{
        Command, ErrorType, Response, Sin,
        grant::{Decree, Gift},
    },
};

use std::{sync::mpsc::Sender, time::SystemTime};

pub fn set(
    terms: &[Vec<u8>],
    temple: &mut Temple,
    tx: Sender<Decree>,
    token: Token,
) -> Result<(), Sin> {
    let terms_len = terms.len();

    if terms_len < 3 {
        tx.send(Decree::Deliver(Gift {
            token,
            response: Response::Error(ErrorType::IncorrectNumberOfArguments(Command::SET)),
        }));

        return Ok(());
    }

    if terms_len == 3 {
        temple.set(
            terms[1].clone(),
            (Value::String(terms[2].clone()), None),
            tx,
            token,
        );
    } else if terms_len == 5 {
        if let Ok(command) = std::str::from_utf8(&terms[3]) {
            if command.to_uppercase() == "EX" {
                if let Ok(expiry) = std::str::from_utf8(&terms[4]) {
                    if let Ok(expiry) = expiry.parse::<u64>() {
                        temple.set(
                            terms[1].clone(),
                            (
                                Value::String(terms[2].clone()),
                                Some(SystemTime::now() + std::time::Duration::from_secs(expiry)),
                            ),
                            tx,
                            token,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
