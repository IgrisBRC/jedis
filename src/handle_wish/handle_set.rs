use std::io::Write;

use mio::net::TcpStream;

use crate::{
    handle_wish::Sin,
    temple::{Temple, Value},
};

pub fn handle_set(
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

            itr += bulk_string_length + 2;

            if itr < buffer_length && read_buffer[itr] == b'$' {
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

                if command.eq_ignore_ascii_case("EX") {
                    itr += bulk_string_length + 2;

                    if read_buffer[itr] == b'$' {
                        itr += 1;

                        let start = itr;

                        while itr < buffer_length && read_buffer[itr] != b'\r' {
                            itr += 1;
                        }

                        let bulk_string_length: usize =
                            std::str::from_utf8(&read_buffer[start..itr])
                                .map_err(|_| Sin::Utf8Error)?
                                .parse()
                                .map_err(|_| Sin::ParseError)?;

                        itr += 2;

                        if let Ok(expiry) =
                            std::str::from_utf8(&read_buffer[itr..itr + bulk_string_length])
                                .map_err(|_| Sin::Utf8Error)?
                                .parse::<u64>()
                        {
                            temple.insert(
                                key.to_string(),
                                (
                                    Value::String(value.into()),
                                    Some(
                                        std::time::SystemTime::now()
                                            + std::time::Duration::from_secs(expiry),
                                    ),
                                ),
                            );

                            stream.write_all(b"+OK\r\n");

                            return Ok(());
                        } else {
                            stream.write_all(b"-ERR Incorrect use of EX\r\n");

                            return Ok(());
                        }
                    }
                } else {
                    stream.write_all(b"-ERR Incorrect use of SET\r\n");

                    return Ok(());
                }
            } else {
                temple.insert(key.to_string(), (Value::String(value.into()), None));

                stream.write_all(b"+OK\r\n");

                return Ok(());
            }
        }
    }

    Err(Sin::Blasphemy)
}
