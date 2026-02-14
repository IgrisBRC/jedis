use std::io::Write;

use mio::net::TcpStream;

use crate::{
    handle_wish::Sin,
    temple::{Temple, Value},
};

pub fn handle_get(
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

        if itr + bulk_string_length + 3 < buffer_length {
            stream.write_all(b"-ERR Invalid Grammar for GET!\r\n");
            return Ok(());
        }

        match temple.get(key.to_string()) {
            Some((Value::String(value), _)) => {
                stream.write_all(format!("${}\r\n", value.len()).as_bytes());
                stream.write_all(
                    format!(
                        "{}\r\n",
                        String::from_utf8(value).map_err(|_| Sin::Utf8Error)?
                    )
                    .as_bytes(),
                );
            }
            Some((_, _)) => {
                todo!()
            }
            None => {
                stream.write_all(b"$-1\r\n");
            }
        }

        return Ok(());
    }

    Err(Sin::Blasphemy)
}
