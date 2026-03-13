// src/tests/integration_test.rs
//
// Integration tests for every command Jerusalem supports.
// Requires a running Jerusalem server on 127.0.0.1:6379.
// Run with: cargo test -- --test-threads=1
//
// All arguments and values are &[u8] / Vec<u8> — no UTF-8 assumption anywhere,
// matching Jerusalem's own binary-safe storage model.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

// ── Connection ────────────────────────────────────────────────────────────────

fn connect() -> TcpStream {
    let s = TcpStream::connect("127.0.0.1:6379")
        .expect("Could not connect to Jerusalem. Is the server running on :6379?");
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    s
}

// ── RESP2 serialiser ──────────────────────────────────────────────────────────

/// Build and send a RESP2 array. Each argument is a raw byte slice.
fn send_command(stream: &mut TcpStream, args: &[&[u8]]) {
    let mut packet = format!("*{}\r\n", args.len()).into_bytes();
    for arg in args {
        let mut header = format!("${}\r\n", arg.len()).into_bytes();
        packet.append(&mut header);
        packet.extend_from_slice(arg);
        packet.extend_from_slice(b"\r\n");
    }
    stream.write_all(&packet).unwrap();
}

/// Read a raw RESP2 response as bytes.
fn read_response(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).unwrap();
    buf.truncate(n);
    buf
}

/// Send a command and return the raw response bytes.
fn cmd(stream: &mut TcpStream, args: &[&[u8]]) -> Vec<u8> {
    send_command(stream, args);
    read_response(stream)
}

// ── RESP2 parser ──────────────────────────────────────────────────────────────

/// Split a byte slice on b"\r\n", returning slices (not copies).
fn split_crlf(buf: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + 1 < buf.len() {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            lines.push(&buf[start..i]);
            start = i + 2;
            i += 2;
        } else {
            i += 1;
        }
    }
    if start < buf.len() {
        lines.push(&buf[start..]);
    }
    lines
}

/// Parse a RESP2 integer line like b":42" -> 42.
fn parse_integer_line(line: &[u8]) -> i64 {
    assert_eq!(line[0], b':', "Expected integer line, got: {:?}", line);
    std::str::from_utf8(&line[1..]).unwrap().parse().unwrap()
}

/// Parse a RESP2 array of bulk strings into Vec<Option<Vec<u8>>>.
/// Null bulk strings ($-1) become None.
fn parse_array(buf: &[u8]) -> Vec<Option<Vec<u8>>> {
    let lines = split_crlf(buf);
    let mut iter = lines.iter();

    let header = iter.next().expect("Empty response");
    assert_eq!(header[0], b'*', "Expected array, got: {:?}", header);
    let count: usize = std::str::from_utf8(&header[1..]).unwrap().parse().unwrap();

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let len_line = iter.next().expect("Truncated array");
        assert_eq!(len_line[0], b'$', "Expected bulk header, got: {:?}", len_line);
        let len: i64 = std::str::from_utf8(&len_line[1..]).unwrap().parse().unwrap();
        if len == -1 {
            result.push(None);
        } else {
            let value = iter.next().expect("Truncated bulk string");
            result.push(Some(value.to_vec()));
        }
    }
    result
}

// ── Assertion helpers ─────────────────────────────────────────────────────────

fn assert_ok(resp: &[u8]) {
    assert_eq!(resp, b"+OK\r\n", "Expected +OK, got: {:?}", resp);
}

fn assert_pong(resp: &[u8]) {
    assert_eq!(resp, b"+PONG\r\n", "Expected +PONG, got: {:?}", resp);
}

fn assert_integer(resp: &[u8], expected: i64) {
    let lines = split_crlf(resp);
    assert_eq!(lines.len(), 1, "Expected single integer line, got: {:?}", resp);
    assert_eq!(
        parse_integer_line(lines[0]),
        expected,
        "Expected :{}, got: {:?}", expected, resp
    );
}

fn assert_bulk(resp: &[u8], expected: &[u8]) {
    let lines = split_crlf(resp);
    assert!(lines.len() >= 2, "Expected bulk string, got: {:?}", resp);
    assert_eq!(lines[0][0], b'$', "Expected bulk header, got: {:?}", lines[0]);
    let len: usize = std::str::from_utf8(&lines[0][1..]).unwrap().parse().unwrap();
    assert_eq!(len, expected.len());
    assert_eq!(lines[1], expected, "Bulk value mismatch");
}

fn assert_null_bulk(resp: &[u8]) {
    assert_eq!(resp, b"$-1\r\n", "Expected null bulk, got: {:?}", resp);
}

fn assert_error(resp: &[u8]) {
    assert_eq!(resp[0], b'-', "Expected error response, got: {:?}", resp);
}

// Convenience: turn a &str literal into &[u8] for use in cmd() calls.
macro_rules! b {
    ($s:expr) => { $s.as_bytes() }
}

// ── PING ──────────────────────────────────────────────────────────────────────

#[test]
fn test_ping() {
    let mut s = connect();
    assert_pong(&cmd(&mut s, &[b!("PING")]));
}

// ── SET / GET ─────────────────────────────────────────────────────────────────

#[test]
fn test_set_and_get() {
    let mut s = connect();
    assert_ok  (&cmd(&mut s, &[b!("SET"), b!("integ:sg:key"), b!("value")]));
    assert_bulk(&cmd(&mut s, &[b!("GET"), b!("integ:sg:key")]), b"value");
}

#[test]
fn test_get_missing_key_returns_null() {
    let mut s = connect();
    assert_null_bulk(&cmd(&mut s, &[b!("GET"), b!("integ:get:missing")]));
}

#[test]
fn test_set_with_ex_expires() {
    let mut s = connect();
    assert_ok  (&cmd(&mut s, &[b!("SET"), b!("integ:ex:key"), b!("temp"), b!("EX"), b!("1")]));
    assert_bulk(&cmd(&mut s, &[b!("GET"), b!("integ:ex:key")]), b"temp");
    std::thread::sleep(Duration::from_secs(2));
    assert_null_bulk(&cmd(&mut s, &[b!("GET"), b!("integ:ex:key")]));
}

// ── APPEND ────────────────────────────────────────────────────────────────────

#[test]
fn test_append() {
    let mut s = connect();
    cmd(&mut s, &[b!("DEL"), b!("integ:app:key")]);
    assert_integer(&cmd(&mut s, &[b!("APPEND"), b!("integ:app:key"), b!("hello")]),    5);
    assert_integer(&cmd(&mut s, &[b!("APPEND"), b!("integ:app:key"), b!(" world")]),  11);
    assert_bulk   (&cmd(&mut s, &[b!("GET"),    b!("integ:app:key")]),                b"hello world");
}

// ── STRLEN ────────────────────────────────────────────────────────────────────

#[test]
fn test_strlen() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:strlen:key"), b!("hello")]);
    assert_integer(&cmd(&mut s, &[b!("STRLEN"), b!("integ:strlen:key")]),     5);
    assert_integer(&cmd(&mut s, &[b!("STRLEN"), b!("integ:strlen:missing")]), 0);
}

// ── INCR / DECR ───────────────────────────────────────────────────────────────

#[test]
fn test_incr_decr() {
    let mut s = connect();
    cmd(&mut s, &[b!("DEL"), b!("integ:counter")]);
    assert_integer(&cmd(&mut s, &[b!("INCR"), b!("integ:counter")]), 1);
    assert_integer(&cmd(&mut s, &[b!("INCR"), b!("integ:counter")]), 2);
    assert_integer(&cmd(&mut s, &[b!("DECR"), b!("integ:counter")]), 1);
}

#[test]
fn test_incr_on_non_integer_returns_error() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:incr:nan"), b!("notanumber")]);
    assert_error(&cmd(&mut s, &[b!("INCR"), b!("integ:incr:nan")]));
}

// ── DEL ───────────────────────────────────────────────────────────────────────

#[test]
fn test_del_returns_count() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:del:a"), b!("1")]);
    cmd(&mut s, &[b!("SET"), b!("integ:del:b"), b!("2")]);
    assert_integer(
        &cmd(&mut s, &[b!("DEL"), b!("integ:del:a"), b!("integ:del:b"), b!("integ:del:missing")]),
        2,
    );
}

// ── EXISTS ────────────────────────────────────────────────────────────────────

#[test]
fn test_exists() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:exists:key"), b!("v")]);
    assert_integer(&cmd(&mut s, &[b!("EXISTS"), b!("integ:exists:key")]),     1);
    assert_integer(&cmd(&mut s, &[b!("EXISTS"), b!("integ:exists:missing")]), 0);
    // Duplicate keys count multiple times per Redis spec
    assert_integer(
        &cmd(&mut s, &[b!("EXISTS"), b!("integ:exists:key"), b!("integ:exists:key")]),
        2,
    );
}

// ── EXPIRE / TTL ──────────────────────────────────────────────────────────────

#[test]
fn test_expire_and_ttl() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:ttl:key"), b!("v")]);
    assert_integer(&cmd(&mut s, &[b!("EXPIRE"), b!("integ:ttl:key"), b!("10")]), 1);

    let resp = cmd(&mut s, &[b!("TTL"), b!("integ:ttl:key")]);
    let ttl = parse_integer_line(split_crlf(&resp)[0]);
    assert!(ttl > 0 && ttl <= 10, "TTL out of range: {}", ttl);
}

#[test]
fn test_expire_missing_key_returns_zero() {
    let mut s = connect();
    assert_integer(&cmd(&mut s, &[b!("EXPIRE"), b!("integ:expire:missing"), b!("10")]), 0);
}

// ── MSET / MGET ───────────────────────────────────────────────────────────────

#[test]
fn test_mset_mget() {
    let mut s = connect();
    assert_ok(&cmd(&mut s, &[
        b!("MSET"),
        b!("integ:m:a"), b!("1"),
        b!("integ:m:b"), b!("2"),
        b!("integ:m:c"), b!("3"),
    ]));
    let r = cmd(&mut s, &[
        b!("MGET"),
        b!("integ:m:a"), b!("integ:m:b"), b!("integ:m:missing"), b!("integ:m:c"),
    ]);
    assert_eq!(parse_array(&r), vec![
        Some(b"1".to_vec()),
        Some(b"2".to_vec()),
        None,
        Some(b"3".to_vec()),
    ]);
}

// ── HSET / HGET / HDEL / HEXISTS / HLEN / HMGET / HGETALL ───────────────────

#[test]
fn test_hash_operations() {
    let mut s = connect();
    let key = b!("integ:hash:user");
    cmd(&mut s, &[b!("DEL"), key]);

    // HSET — two new fields
    assert_integer(&cmd(&mut s, &[b!("HSET"), key, b!("name"), b!("alice"), b!("age"), b!("30")]), 2);
    // Updating an existing field returns 0 new additions
    assert_integer(&cmd(&mut s, &[b!("HSET"), key, b!("name"), b!("bob")]), 0);

    // HGET
    assert_bulk     (&cmd(&mut s, &[b!("HGET"), key, b!("name")]),    b"bob");
    assert_null_bulk(&cmd(&mut s, &[b!("HGET"), key, b!("missing")]));

    // HEXISTS
    assert_integer(&cmd(&mut s, &[b!("HEXISTS"), key, b!("name")]), 1);
    assert_integer(&cmd(&mut s, &[b!("HEXISTS"), key, b!("nope")]),  0);

    // HLEN
    assert_integer(&cmd(&mut s, &[b!("HLEN"), key]), 2);

    // HMGET — order matches requested field order
    assert_eq!(
        parse_array(&cmd(&mut s, &[b!("HMGET"), key, b!("name"), b!("age"), b!("missing")])),
        vec![Some(b"bob".to_vec()), Some(b"30".to_vec()), None],
    );

    // HDEL
    assert_integer(&cmd(&mut s, &[b!("HDEL"), key, b!("age"), b!("nope")]), 1);
    assert_integer(&cmd(&mut s, &[b!("HLEN"), key]), 1);

    // HGETALL — flat interleaved [field, value, ...]; only "name"->"bob" remains
    let flat = parse_array(&cmd(&mut s, &[b!("HGETALL"), key]));
    assert_eq!(flat.len(), 2, "Expected [field, value], got: {:?}", flat);
    assert_eq!(flat[0], Some(b"name".to_vec()));
    assert_eq!(flat[1], Some(b"bob".to_vec()));
}

// ── LPUSH / RPUSH / LPOP / RPOP / LLEN / LRANGE / LINDEX / LSET / LREM ──────

#[test]
fn test_list_operations() {
    let mut s = connect();
    let key = b!("integ:list:l");
    cmd(&mut s, &[b!("DEL"), key]);

    // RPUSH
    assert_integer(&cmd(&mut s, &[b!("RPUSH"), key, b!("a"), b!("b"), b!("c")]), 3);

    // LLEN
    assert_integer(&cmd(&mut s, &[b!("LLEN"), key]), 3);

    // LRANGE — list order is deterministic
    assert_eq!(
        parse_array(&cmd(&mut s, &[b!("LRANGE"), key, b!("0"), b!("-1")])),
        vec![Some(b"a".to_vec()), Some(b"b".to_vec()), Some(b"c".to_vec())],
    );

    // LINDEX
    assert_bulk     (&cmd(&mut s, &[b!("LINDEX"), key, b!("0")]),  b"a");
    assert_bulk     (&cmd(&mut s, &[b!("LINDEX"), key, b!("-1")]), b"c");
    assert_null_bulk(&cmd(&mut s, &[b!("LINDEX"), key, b!("99")]));

    // LSET
    assert_ok  (&cmd(&mut s, &[b!("LSET"),   key, b!("1"), b!("B")]));
    assert_bulk(&cmd(&mut s, &[b!("LINDEX"), key, b!("1")]), b"B");

    // LPUSH prepends
    assert_integer(&cmd(&mut s, &[b!("LPUSH"),  key, b!("z")]), 4);
    assert_bulk   (&cmd(&mut s, &[b!("LINDEX"), key, b!("0")]), b"z");

    // LPOP / RPOP
    assert_bulk(&cmd(&mut s, &[b!("LPOP"), key]), b"z");
    assert_bulk(&cmd(&mut s, &[b!("RPOP"), key]), b"c");

    // LREM
    cmd(&mut s, &[b!("DEL"), key]);
    cmd(&mut s, &[b!("RPUSH"), key, b!("x"), b!("y"), b!("x"), b!("z"), b!("x")]);
    assert_integer(&cmd(&mut s, &[b!("LREM"), key, b!("2"), b!("x")]), 2);
    assert_integer(&cmd(&mut s, &[b!("LLEN"), key]), 3);
}

#[test]
fn test_lpop_rpop_with_count() {
    let mut s = connect();
    let key = b!("integ:list:popcount");
    cmd(&mut s, &[b!("DEL"), key]);
    cmd(&mut s, &[b!("RPUSH"), key, b!("a"), b!("b"), b!("c"), b!("d")]);

    assert_eq!(
        parse_array(&cmd(&mut s, &[b!("LPOP"), key, b!("2")])),
        vec![Some(b"a".to_vec()), Some(b"b".to_vec())],
    );
    assert_eq!(
        parse_array(&cmd(&mut s, &[b!("RPOP"), key, b!("2")])),
        vec![Some(b"d".to_vec()), Some(b"c".to_vec())],
    );
}

// ── SADD / SREM / SISMEMBER / SMEMBERS ───────────────────────────────────────

#[test]
fn test_set_operations() {
    let mut s = connect();
    let key = b!("integ:set:s");
    cmd(&mut s, &[b!("DEL"), key]);

    // SADD
    assert_integer(&cmd(&mut s, &[b!("SADD"), key, b!("a"), b!("b"), b!("c")]), 3);
    assert_integer(&cmd(&mut s, &[b!("SADD"), key, b!("a")]), 0); // duplicate

    // SISMEMBER
    assert_integer(&cmd(&mut s, &[b!("SISMEMBER"), key, b!("a")]), 1);
    assert_integer(&cmd(&mut s, &[b!("SISMEMBER"), key, b!("z")]), 0);
    //
    // // SMEMBERS — HashSet order is non-deterministic so sort before asserting
    let mut members: Vec<Vec<u8>> = parse_array(&cmd(&mut s, &[b!("SMEMBERS"), key]))
        .into_iter()
        .flatten()
        .collect();
    members.sort();
    assert_eq!(members, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
    //
    // // SREM
    assert_integer(&cmd(&mut s, &[b!("SREM"), key, b!("a"), b!("missing")]), 1);
    assert_integer(&cmd(&mut s, &[b!("SISMEMBER"), key, b!("a")]), 0);
}

// ── WRONGTYPE errors ──────────────────────────────────────────────────────────

#[test]
fn test_wrongtype_errors() {
    let mut s = connect();
    cmd(&mut s, &[b!("SET"), b!("integ:wt:str"), b!("v")]);
    assert_error(&cmd(&mut s, &[b!("LPUSH"), b!("integ:wt:str"), b!("x")]));
    assert_error(&cmd(&mut s, &[b!("HSET"),  b!("integ:wt:str"), b!("f"), b!("v")]));
    assert_error(&cmd(&mut s, &[b!("SADD"),  b!("integ:wt:str"), b!("m")]));
}
