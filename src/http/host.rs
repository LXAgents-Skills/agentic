//! The `Host` header allow-list.
//!
//! Runs ahead of every route, `/healthz` included: an allow-list that guards `/mcp` and not
//! `/healthz` has a hole in it. With no list configured, a tokenless loopback server still gets
//! the localhost list, because then the `Host` check is what stops a web page that rebinds its
//! own name to 127.0.0.1 from reading this server through the browser.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use super::rpc_error::rpc_error;

/// The hostnames a tokenless loopback server answers to.
pub const LOCALHOST_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The hostname in a `Host` header: lowercased, port dropped, IPv6 kept in brackets.
pub fn hostname(host: &str) -> String {
    let host = host.trim().to_lowercase();
    if host.starts_with('[') {
        return match host.find(']') {
            Some(end) => host[..=end].to_string(),
            None => host,
        };
    }
    match host.rsplit_once(':') {
        Some((name, port)) if !name.contains(':') && port.bytes().all(|b| b.is_ascii_digit()) => {
            name.to_string()
        }
        _ => host,
    }
}

/// The hostnames this server answers to, shared by every request.
#[derive(Clone)]
pub struct AllowedHosts(pub Arc<Vec<String>>);

/// Middleware: refuse a request whose `Host` is missing or not on the list, with a 403.
pub async fn require_allowed_host(
    State(AllowedHosts(allowed)): State<AllowedHosts>,
    request: Request,
    next: Next,
) -> Response {
    let Some(host) = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(hostname)
    else {
        return rpc_error(StatusCode::FORBIDDEN, -32000, "Missing Host header");
    };
    if !allowed.iter().any(|name| hostname(name) == host) {
        return rpc_error(StatusCode::FORBIDDEN, -32000, format!("Invalid Host: {host}"));
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_port_is_dropped_and_ipv6_keeps_its_brackets() {
        assert_eq!(hostname("LocalHost:3000"), "localhost");
        assert_eq!(hostname("127.0.0.1"), "127.0.0.1");
        assert_eq!(hostname("[::1]:3000"), "[::1]");
        assert_eq!(hostname("evil.example"), "evil.example");
    }
}
