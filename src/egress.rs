use std::io::Write;
use jerusalem::wish::{grant::Gift, InfoType, Response};
use mio::net::TcpStream;

pub fn egress(stream: &mut TcpStream, gift: Gift) {
    match gift.response {
        Response::Info(InfoType::Ok) => {
            stream.write_all(b"+OK\r\n").ok();
        }
        Response::Info(InfoType::Pong) => {
            stream.write_all(b"+PONG\r\n").ok();
        }
        _ => {}
    }
}
