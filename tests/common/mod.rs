//! Helpers shared by the integration tests. Each test file includes this module and uses
//! only part of it.
#![allow(dead_code)]

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use lxagents_agentic::{instructions::surface::instructions, server::AgenticServer};
use rmcp::{RoleClient, ServiceExt, service::RunningService};

/// The built binary.
pub const BIN: &str = env!("CARGO_BIN_EXE_lxagents-agentic");

/// The repository's `content/` on disk.
pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("content")
}

/// Every `.md` under `content/`, as set-relative forward-slash paths, sorted.
pub fn markdown_files() -> Vec<String> {
    fn walk(dir: &Path, base: &Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("read content/") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(&path, base, found);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                let relative = path.strip_prefix(base).expect("under content/");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut found = Vec::new();
    walk(&content_dir(), &content_dir(), &mut found);
    found.sort();
    found
}

/// A server over the embedded set.
pub fn server() -> AgenticServer {
    AgenticServer::new(instructions().expect("content/ builds"))
}

/// An in-memory client connected to a fresh server.
pub async fn client() -> RunningService<RoleClient, ()> {
    let (client_io, server_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        let running = server()
            .serve(tokio::io::split(server_io))
            .await
            .expect("server handshake");
        let _ = running.waiting().await;
    });
    ().serve(tokio::io::split(client_io))
        .await
        .expect("client handshake")
}

/// A port nothing is listening on.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

/// A raw HTTP/1.1 response: status, headers (lowercased names), body.
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Send one request with exactly these headers, and read the whole response.
///
/// Raw on purpose: an HTTP client library would add or refuse to forge a `Host` header.
pub fn request(port: u16, method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Reply {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    if !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("host")) {
        head.push_str(&format!("Host: 127.0.0.1:{port}\r\n"));
    }
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).expect("write head");
    stream.write_all(body.as_bytes()).expect("write body");

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("read");
    let raw = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| line.split(' ').nth(1))
        .and_then(|code| code.parse().ok())
        .expect("status line");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_lowercase(), value.trim().to_string()))
        .collect::<Vec<_>>();
    let chunked = headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.contains("chunked"));
    let body = if chunked { dechunk(body) } else { body.to_string() };
    Reply {
        status,
        headers,
        body,
    }
}

fn dechunk(mut body: &str) -> String {
    let mut out = String::new();
    while let Some((size, rest)) = body.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size.trim(), 16) else {
            break;
        };
        if size == 0 || rest.len() < size {
            break;
        }
        out.push_str(&rest[..size]);
        body = rest[size..].trim_start_matches("\r\n");
    }
    out
}

/// A JSON-RPC `tools/list` POST body and the headers a Streamable HTTP client sends.
pub const LIST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
pub const JSON_HEADERS: [(&str, &str); 2] = [
    ("Content-Type", "application/json"),
    ("Accept", "application/json, text/event-stream"),
];

/// The binary, started with `MCP_TRANSPORT=http`, one worker thread, and `env`.
pub fn spawn_http(env: &[(&str, &str)]) -> Child {
    let mut command = Command::new(BIN);
    command
        .env_clear()
        .env("MCP_TRANSPORT", "http")
        .env("MCP_CLUSTER_WORKERS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    for (key, value) in env {
        command.env(key, value);
    }
    command.spawn().expect("spawn the binary")
}

/// Wait until `port` accepts connections, or panic after ten seconds.
pub fn wait_for_port(port: u16) {
    let start = Instant::now();
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "nothing listened on {port}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Send SIGTERM, wait for exit, and return everything the child wrote to stderr.
#[cfg_attr(unix, allow(unused_mut))]
pub fn stop(mut child: Child) -> String {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(child.id().to_string())
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let output = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&output.stderr).into_owned()
}
