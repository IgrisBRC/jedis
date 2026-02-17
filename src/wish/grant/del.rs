use crate::{temple::{Temple, Value}, wish::Sin};
use mio::net::TcpStream;
use std::{io::Write, sync::mpsc::{Receiver, Sender}, time::SystemTime};

pub fn del(terms: &[Vec<u8>], stream: &mut TcpStream, temple: &mut Temple,
    tx: Sender<Option<(Value, Option<SystemTime>)>>,
    rx: &Receiver<Option<(Value, Option<SystemTime>)>>,
) -> Result<(), Sin> {
    if terms.len() < 2 {
        stream
            .write_all(b"-ERR wrong number of arguments for DEL command\r\n")
            .map_err(|_| Sin::Disconnected)?;
        return Ok(());
    }

    let mut deleted_count = 0;

    for key_bytes in &terms[1..] {
        let key = std::str::from_utf8(key_bytes).map_err(|_| Sin::Utf8Error)?;
        if temple.remove(key.into(), tx.clone(), &rx).is_some() {
            deleted_count += 1;
        }
    }

    let response = format!(":{}\r\n", deleted_count);
    stream
        .write_all(response.as_bytes())
        .map_err(|_| Sin::Disconnected)?;

    Ok(())
}
