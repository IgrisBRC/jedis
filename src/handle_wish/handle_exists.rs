use std::io::Write;

use mio::net::TcpStream;

use crate::{handle_wish::Sin, temple::Temple};

pub fn handle_exists(
    read_buffer: &mut [u8],
    mut itr: usize,
    buffer_length: usize,
    number_of_terms: usize,
    temple: &mut Temple,
    stream: &mut TcpStream,
) -> Result<(), Sin> {
    let mut keys_found = 0;

    for _ in 0..(number_of_terms - 1) {
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

            let key = std::str::from_utf8(&read_buffer[itr..itr+bulk_string_length])
                 .map_err(|_| Sin::Utf8Error)?;

            if let Some(_) = temple.get(key.to_string()) {
                keys_found += 1;
            }

            itr += bulk_string_length + 2;
        } else {
            return Err(Sin::Blasphemy);
        }
    }

    stream.write_all(format!(":{}\r\n", keys_found).as_bytes());

    Ok(())
}
