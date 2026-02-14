use mio::net::TcpStream;
use std::io::{ErrorKind, Read, Write};

use crate::temple::Temple;

pub struct Virtue {
    operation: String, //Will defo make an enum for this, but right now it's okay
    terms: Vec<Vec<u8>>,
    backlog: bool,
}

pub struct Pilgrim {
    pub stream: TcpStream,
    pub virtue: Option<Virtue>,
}

pub enum Sin {
    Utf8Error,
    ParseError,
    ClientDisconnected,
    Blasphemy,
}

mod handle_append;
mod handle_decr;
mod handle_del;
mod handle_exists;
mod handle_get;
mod handle_incr;
mod handle_set;

pub fn handle_wish(pilgrim: &mut Pilgrim, mut temple: Temple) -> Result<(), Sin> {
    let read_buffer: &mut [u8] = &mut [0; 1024];

    match pilgrim.stream.read(read_buffer) {
        Ok(0) => {
            return Err(Sin::ClientDisconnected)
        }
        Ok(buffer_length) => {
            if read_buffer[0] == b'*' {
                let mut itr = 1;

                while itr < buffer_length && read_buffer[itr] != b'\r' {
                    itr += 1;
                }

                let number_of_terms: usize = std::str::from_utf8(&read_buffer[1..itr])
                    .map_err(|_| Sin::Utf8Error)?
                    .parse()
                    .map_err(|_| Sin::ParseError)?;

                itr += 2;

                if read_buffer[itr] == b'$' {
                    itr += 1;
                    let start = itr;

                    while itr < buffer_length && read_buffer[itr] != b'\r' {
                        itr += 1;
                    }

                    let bulk_string_length: usize = std::str::from_utf8(&read_buffer[start..itr])
                        .map_err(|_| Sin::Utf8Error)?
                        .parse()
                        .map_err(|_| Sin::ParseError)?;

                    itr += 2;

                    let command = std::str::from_utf8(&read_buffer[itr..itr + bulk_string_length])
                            .map_err(|_| Sin::Utf8Error)?;

                    match command.to_uppercase().as_str() {
                        "PING" => {
                            pilgrim.stream.write_all(b"+PONG!\r\n");
                            return Ok(());
                        }
                        "SET" => handle_set::handle_set(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "GET" => handle_get::handle_get(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "DEL" => handle_del::handle_del(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            number_of_terms,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "EXISTS" => handle_exists::handle_exists(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            number_of_terms,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "APPEND" => handle_append::handle_append(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "INCR" => handle_incr::handle_incr(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        "DECR" => handle_decr::handle_decr(
                            read_buffer,
                            itr + bulk_string_length + 2,
                            buffer_length,
                            &mut temple,
                            &mut pilgrim.stream,
                        )?,
                        _ => {
                            pilgrim.stream.write_all(b"-ERR Unknown Command!\r\n");
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(e) => {
            if e.kind() == ErrorKind::WouldBlock {
                return Ok(());
            } else {
                eprint!("{}", e);
                return Err(Sin::ClientDisconnected);
            }
        }
    }

    Ok(())
}
