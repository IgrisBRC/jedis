use crate::{
    handle_wish::Sin,
    temple::{Temple, Value},
};
use mio::net::TcpStream;
use std::{io::Write, str::from_utf8};

pub fn handle_append(
    read_buffer: &mut [u8],
    mut itr: usize,
    buffer_length: usize,
    temple: &mut Temple,
    stream: &mut TcpStream,
) -> Result<(), Sin> {
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

        let key = std::str::from_utf8(&read_buffer[itr..itr + bulk_string_length])
            .map_err(|_| Sin::Utf8Error)?;

        itr += bulk_string_length + 2;

        if itr >= buffer_length {
            stream.write_all(b"-ERR Invalid Grammar!\r\n");
            return Ok(());
        }

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

            let value = std::str::from_utf8(&read_buffer[itr..itr + bulk_string_length])
                .map_err(|_| Sin::Utf8Error)?;

            let mut value_length = value.len();

            if let Some((Value::String(mut existing_value), _)) = temple.get(key.to_string()) {
                existing_value.append(&mut value.as_bytes().to_vec());
                value_length = existing_value.len();
                temple.insert(key.to_string(), (Value::String(existing_value), None));
            } else {
                temple.insert(key.to_string(), (Value::String(value.into()), None));
            }

            stream.write_all(format!(":{}\r\n", value_length).as_bytes());

            return Ok(());
        }
    }

    Err(Sin::Blasphemy)
}
