use crate::{
    handle_wish::Sin,
    temple::{Temple, Value},
};
use mio::net::TcpStream;
use std::io::Write;

pub fn handle_incr(
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
            stream.write_all(b"-ERR Invalid Grammar for INCR!\r\n");
            return Ok(());
        }

        match temple.get(key.to_string()) {
            Some((Value::String(value), _)) => {
                if let Ok(value) = String::from_utf8(value)
                    .map_err(|_| Sin::Utf8Error)?
                    .parse::<i64>()
                {
                    let incremented_value = value + 1;

                    temple.insert(
                        key.to_string(),
                        (Value::String(incremented_value.to_string().into()), None),
                    );

                    stream.write_all(format!(":{}\r\n", incremented_value).as_bytes());
                } else {
                    stream.write_all(b"-ERR Invalid value for INCR\r\n");
                }
            }
            Some((_, _)) => {
                stream.write_all(b"-Invalid use of INCR\r\n");
            }

            None => {
                temple.insert(key.to_string(), (Value::String(1.to_string().into()), None));
                stream.write_all(b":1\r\n");
            }
        }

        return Ok(());
    }

    Err(Sin::Blasphemy)
}
