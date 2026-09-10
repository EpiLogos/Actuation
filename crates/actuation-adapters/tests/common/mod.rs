#![allow(dead_code)]
use actuation_adapters::{effects::NativeEffects, DetectionOptions, NativeCatalog};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
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
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = vec![];
            while !request.ends_with(b"\r\n\r\n") {
                let mut b = [0; 1];
                assert_eq!(stream.read(&mut b).unwrap(), 1);
                request.extend_from_slice(&b);
                assert!(request.len() < 16384);
            }
            requests.push(String::from_utf8(request).unwrap());
            write!(stream,"HTTP/1.1 {code} Controlled\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
        requests
    });
    (url, handle)
}
