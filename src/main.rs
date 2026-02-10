use std::net::TcpListener;

use mini_redis::choir::Choir;
use mini_redis::handle_connection::handle_connection;
use mini_redis::temple::Temple;

fn main() {
    let temple = Temple::new(String::from("IgrisDB"));
    let choir = Choir::new(6);

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("mini_redis is alive and listening on port 6379");

    for stream in listener.incoming() {
        let sanctum = temple.sanctify();

        match stream {
            Ok(s) => choir.sing(move || match handle_connection(s, sanctum) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e)
                }
            }),
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
