use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

#[test]
fn test_concurrent_increments() {

    let n_clients = 10;
    let increments_per_client = 1000;
    let mut handles = vec![];

    for _ in 0..n_clients {
        handles.push(thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
            for _ in 0..increments_per_client {
                let cmd = "*2\r\n$4\r\nINCR\r\n$6\r\nhammer\r\n";
                stream.write_all(cmd.as_bytes()).unwrap();

                let mut buf = [0; 128];
                let _ = stream.read(&mut buf).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$6\r\nhammer\r\n")
        .unwrap();

    let mut buf = [0; 128];
    let n = stream.read(&mut buf).unwrap();
    let final_val = std::str::from_utf8(&buf[..n]).unwrap();

    assert!(final_val.contains("10000"));
    println!("The Temple held firm: {}", final_val);
}
