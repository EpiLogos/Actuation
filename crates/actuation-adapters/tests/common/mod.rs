#![allow(dead_code)]
use actuation_adapters::{effects::NativeEffects, DetectionOptions, NativeCatalog};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
    time::{Duration, Instant},
};
pub fn effects(root: &Path) -> NativeEffects {
    NativeEffects::new(
        Some(root.into()),
        root.into(),
        vec![root.join("bin")],
        BTreeMap::new(),
    )
}
pub fn options(catalog: &NativeCatalog) -> DetectionOptions {
    DetectionOptions {
        observed_at: serde_json::from_value(json!("2026-09-10T12:00:00Z")).unwrap(),
        probe_versions: false,
        catalog_revision: catalog.revision(),
    }
}
pub fn catalog_value() -> Value {
    serde_json::from_str(include_str!("../../../../catalog/targets.json")).unwrap()
}
#[cfg(unix)]
pub fn script(path: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn http_request_headers(stream: &mut TcpStream, timeout: Duration) -> io::Result<String> {
    // Accepted sockets can retain the nonblocking listener's mode. A read
    // timeout alone does not make such a socket wait for the next fragment.
    stream.set_nonblocking(false)?;
    let started = Instant::now();
    let mut request = vec![];
    while !request.ends_with(b"\r\n\r\n") {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "fixture header deadline"))?;
        stream.set_read_timeout(Some(remaining))?;
        let mut byte = [0; 1];
        match stream.read(&mut byte) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "fixture request ended before complete headers",
                ));
            }
            Ok(_) => request.extend_from_slice(&byte),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
        if request.len() >= 16384 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "fixture request headers exceed bound",
            ));
        }
    }
    String::from_utf8(request).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
/// A bounded controlled native HTTP peer, not provider evidence. It records
/// requests and returns the supplied protocol specimens over real sockets.
pub fn http_peer(responses: Vec<(u16, String)>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut requests = vec![];
        for (code, body) in responses {
            let start = Instant::now();
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(peer) => break peer,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && start.elapsed() < Duration::from_secs(5) =>
                    {
                        thread::sleep(Duration::from_millis(2))
                    }
                    other => panic!("fixture peer failed to receive expected request: {other:?}"),
                }
            };
            requests.push(
                http_request_headers(&mut stream, Duration::from_secs(2))
                    .expect("fixture peer could not read bounded HTTP request"),
            );
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            write!(stream,"HTTP/1.1 {code} Controlled\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
        requests
    });
    (url, handle)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn connected_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        // Reproduce the inherited mode on every OS, not only on macOS.
        server.set_nonblocking(true).unwrap();
        (server, client)
    }

    #[test]
    fn nonblocking_peer_waits_for_delayed_fragmented_headers() {
        let (mut server, mut client) = connected_pair();
        client
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let writer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            client
                .write_all(b"GET /api/tags HTTP/1.1\r\nHost: fixture\r\n")
                .unwrap();
            thread::sleep(Duration::from_millis(20));
            client.write_all(b"\r\n").unwrap();
        });
        let request = http_request_headers(&mut server, Duration::from_secs(2)).unwrap();
        writer.join().unwrap();
        assert_eq!(request, "GET /api/tags HTTP/1.1\r\nHost: fixture\r\n\r\n");
    }

    #[test]
    fn idle_peer_keeps_a_finite_header_deadline() {
        let (mut server, _client) = connected_pair();
        let start = Instant::now();
        let error = http_request_headers(&mut server, Duration::from_millis(30)).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn incomplete_headers_are_not_a_valid_request() {
        let (mut server, mut client) = connected_pair();
        client.write_all(b"GET / HTTP/1.1\r\n").unwrap();
        drop(client);
        let error = http_request_headers(&mut server, Duration::from_secs(2)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }
}
