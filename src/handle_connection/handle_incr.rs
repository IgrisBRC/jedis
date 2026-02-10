use std::{
    io::{BufReader, BufWriter, Lines},
    net::TcpStream,
};

use crate::handle_connection::util;
use crate::temple::Temple;

pub fn handle_incr(
    db: &Temple,
    reader_lines: &mut Lines<BufReader<&TcpStream>>,
    count: usize,
    wstream: &mut BufWriter<&TcpStream>,
    count_ledger: &mut i32,
) -> Result<(), String> {
    if count != 2 {
        util::write_to_wstream(wstream, b"-ERR Protocol Error\r\n")?;
        util::cleanup(count_ledger, reader_lines);
        return Ok(());
    }

    let key = match util::validate_and_get_next_term(reader_lines, count_ledger) {
        Ok(t) => t,
        Err(e) => {
            util::write_to_wstream(wstream, format!("{}\r\n", e).as_bytes())?;
            return Ok(());
        }
    };

    if let Some(value) = db.incr(key.clone()) {
        util::write_to_wstream(wstream, format!(":{}\r\n", value).as_bytes())?;
    } else {
        util::write_to_wstream(
            wstream,
            b"-ERR invalid use of INCR, value not in number form.\r\n",
        )?;
    }

    Ok(())
}
