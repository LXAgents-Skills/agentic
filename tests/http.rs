//! The HTTP transport: the app in-process, and the binary's refusal to run open.

mod common;

use common::{JSON_HEADERS, LIST, free_port, request, server, spawn_http, stop, wait_for_port};
use lxagents_agentic::{
    http::app::{AppOptions, BODY_LIMIT_BYTES, router},
    identity::{SERVER_ID, tool_id},
    version::VERSION,
};

const TOKEN: &str = "tttttttttttttttttttttttttttttttt";

/// Serve `options` on a free loopback port on a background runtime, and return the port.
fn start(options: AppOptions) -> u16 {
    let app = router(server(), options).expect("the app builds");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("listener");
            axum::serve(listener, app).await.expect("serve");
        });
    });
    port
}

fn with_token() -> AppOptions {
    AppOptions {
        token: TOKEN.into(),
        allowed_hosts: vec![],
        local: false,
    }
}

fn tokenless_loopback() -> AppOptions {
    AppOptions {
        token: String::new(),
        allowed_hosts: vec![],
        local: true,
    }
}

#[test]
fn off_loopback_the_app_cannot_be_built_without_a_token() {
    let error = router(
        server(),
        AppOptions {
            token: String::new(),
            allowed_hosts: vec![],
            local: false,
        },
    )
    .unwrap_err();
    assert!(error.contains("MCP_AUTH_TOKEN is not set"), "{error}");
    assert!(
        router(server(), AppOptions::default()).is_err(),
        "the default options must not build an open app"
    );
}

#[test]
fn a_token_shorter_than_32_characters_is_refused_even_on_loopback() {
    let error = router(
        server(),
        AppOptions {
            token: "short".into(),
            allowed_hosts: vec![],
            local: true,
        },
    )
    .unwrap_err();
    assert!(error.contains("at least 32"), "{error}");
}

#[test]
fn with_a_token_mcp_needs_it_and_healthz_does_not() {
    let port = start(with_token());

    let health = request(port, "GET", "/healthz", &[], "");
    assert_eq!(health.status, 200);
    let body: serde_json::Value = serde_json::from_str(&health.body).expect("json");
    assert_eq!(
        body,
        serde_json::json!({ "status": "ok", "server": SERVER_ID, "version": VERSION })
    );

    let missing = request(port, "POST", "/mcp", &JSON_HEADERS, LIST);
    assert_eq!(missing.status, 401);
    assert_eq!(missing.header("www-authenticate"), Some(r#"Bearer realm="mcp""#));
    assert!(missing.body.contains("-32001"), "{}", missing.body);

    let wrong_header = format!("Bearer {}", "x".repeat(32));
    let wrong = request(
        port,
        "POST",
        "/mcp",
        &[JSON_HEADERS[0], JSON_HEADERS[1], ("Authorization", &wrong_header)],
        LIST,
    );
    assert_eq!(wrong.status, 401);
    assert!(
        wrong
            .header("www-authenticate")
            .unwrap_or_default()
            .contains("invalid_token")
    );

    let good_header = format!("Bearer {TOKEN}");
    let ok = request(
        port,
        "POST",
        "/mcp",
        &[JSON_HEADERS[0], JSON_HEADERS[1], ("Authorization", &good_header)],
        LIST,
    );
    assert_eq!(ok.status, 200, "{}", ok.body);
    assert!(ok.body.contains(&tool_id("automation")), "{}", ok.body);
}

#[test]
fn with_a_token_routes_are_not_revealed_to_a_caller_without_it() {
    let port = start(with_token());
    assert_eq!(request(port, "GET", "/nope", &[], "").status, 401);
    assert_eq!(request(port, "GET", "/mcp", &[], "").status, 401);
    assert_eq!(
        request(port, "HEAD", "/healthz", &[], "").status,
        401,
        "the exemption is exactly GET /healthz"
    );
    assert_eq!(
        request(port, "GET", "/healthz/", &[], "").status,
        401,
        "the exemption is exactly GET /healthz"
    );
}

#[test]
fn on_loopback_without_a_token_only_localhost_host_headers_are_answered() {
    let port = start(tokenless_loopback());
    assert_eq!(request(port, "POST", "/mcp", &JSON_HEADERS, LIST).status, 200);
    let evil = request(port, "GET", "/healthz", &[("Host", "evil.example")], "");
    assert_eq!(evil.status, 403);
    assert!(evil.body.contains("-32000"), "{}", evil.body);
    assert_eq!(
        request(port, "GET", "/healthz", &[("Host", "localhost")], "").status,
        200
    );
    assert_eq!(
        request(port, "GET", "/healthz", &[("Host", "localhost:3000")], "").status,
        200
    );
    assert_eq!(
        request(port, "GET", "/healthz", &[("Host", "[::1]:3000")], "").status,
        200
    );
}

#[test]
fn a_configured_allow_list_applies_with_a_token_too() {
    let port = start(AppOptions {
        token: TOKEN.into(),
        allowed_hosts: vec!["mcp.example".into()],
        local: false,
    });
    assert_eq!(
        request(port, "GET", "/healthz", &[("Host", "mcp.example")], "").status,
        200
    );
    assert_eq!(
        request(port, "GET", "/healthz", &[("Host", "127.0.0.1")], "").status,
        403
    );
}

#[test]
fn unknown_routes_are_404_and_other_methods_on_mcp_are_405() {
    let port = start(tokenless_loopback());
    let missing = request(port, "GET", "/nope?x=1", &[], "");
    assert_eq!(missing.status, 404);
    assert!(missing.body.contains("Not found: /nope?x=1"), "{}", missing.body);
    let method = request(port, "GET", "/mcp", &[], "");
    assert_eq!(method.status, 405);
    assert!(
        method.body.contains("GET is not supported in stateless mode"),
        "{}",
        method.body
    );
    assert_eq!(request(port, "DELETE", "/mcp", &[], "").status, 405);
    assert_eq!(request(port, "POST", "/healthz", &[], "").status, 404);
}

#[test]
fn a_malformed_or_oversized_body_is_a_parse_error() {
    let port = start(tokenless_loopback());
    let bad = request(port, "POST", "/mcp", &JSON_HEADERS, "{bad");
    assert_eq!(bad.status, 400);
    assert!(bad.body.contains("-32700"), "{}", bad.body);

    let huge = format!("\"{}\"", "a".repeat(BODY_LIMIT_BYTES + 1));
    let oversized = request(port, "POST", "/mcp", &JSON_HEADERS, &huge);
    assert_eq!(oversized.status, 400);
    assert!(oversized.body.contains("-32700"), "{}", oversized.body);
}

#[test]
fn no_response_names_the_framework() {
    let port = start(tokenless_loopback());
    let health = request(port, "GET", "/healthz", &[], "");
    assert_eq!(health.header("x-powered-by"), None);
    assert_eq!(health.header("server"), None);
}

#[test]
fn the_binary_refuses_to_start_on_all_interfaces_without_a_token() {
    let port = free_port().to_string();
    let child = spawn_http(&[("HOST", "0.0.0.0"), ("PORT", &port)]);
    let output = child.wait_with_output().expect("wait");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("will not start the HTTP transport: MCP_AUTH_TOKEN is not set"),
        "{stderr}"
    );
}

#[test]
fn the_binary_starts_on_loopback_without_a_token_and_drains_on_sigterm() {
    let port = free_port();
    let child = spawn_http(&[("HOST", "127.0.0.1"), ("PORT", &port.to_string())]);
    wait_for_port(port);
    let health = request(port, "GET", "/healthz", &[], "");
    assert_eq!(health.status, 200);
    let stderr = stop(child);
    assert!(stderr.contains("serving over http"), "{stderr}");
    assert!(
        stderr.contains("bound to loopback, so no bearer token is required"),
        "{stderr}"
    );
}

#[test]
fn the_binary_serves_off_loopback_with_a_token() {
    let port = free_port();
    let child = spawn_http(&[
        ("HOST", "0.0.0.0"),
        ("PORT", &port.to_string()),
        ("MCP_AUTH_TOKEN", TOKEN),
    ]);
    wait_for_port(port);
    assert_eq!(request(port, "GET", "/healthz", &[], "").status, 200);
    assert_eq!(request(port, "POST", "/mcp", &JSON_HEADERS, LIST).status, 401);
    let stderr = stop(child);
    assert!(
        stderr.contains("bearer token required on every route except GET /healthz"),
        "{stderr}"
    );
    assert!(stderr.contains("MCP_ALLOWED_HOSTS is unset"), "{stderr}");
    assert!(!stderr.contains(TOKEN), "the token must never be logged");
}
