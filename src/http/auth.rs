//! Bearer-token authentication for the HTTP transport.
//!
//! **The transport decides whether a token is needed.** stdio is a pipe that a client spawns on
//! its own machine, so there is nobody to authenticate. HTTP is a socket anyone who can reach
//! the port can open, so every request must prove the caller holds the token.
//!
//! **Where the socket listens decides whether that is a network.** A server bound to loopback
//! can only be reached from this machine, so a token is optional there, and the localhost
//! `Host` allow-list stops a web page reaching it by DNS rebinding. Any other bind is a server,
//! and it refuses to start without a token. A token that is set is always enforced.

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::rpc_error::rpc_error;

/// The shortest token the server will accept, in characters. `openssl rand -hex 16` produces
/// at least this, and the documented `openssl rand -hex 32` produces 64.
pub const MIN_TOKEN_LENGTH: usize = 32;

/// The configured token, or an empty string when there is none.
///
/// Read at call time and never captured at startup. Surrounding whitespace is dropped, because
/// a token copied out of a file often carries a trailing newline and an HTTP header cannot.
pub fn configured_token() -> String {
    std::env::var("MCP_AUTH_TOKEN")
        .map(|token| token.trim().to_string())
        .unwrap_or_default()
}

/// Why `token` cannot be used, or `None` when it can. Names the variable, never the value.
///
/// `required: false` is the local case: no token is then fine, but a token that is set and
/// too short is still a problem, because it was meant to protect something and would not.
pub fn token_problem(token: &str, required: bool) -> Option<String> {
    if token.is_empty() {
        return required.then(|| "MCP_AUTH_TOKEN is not set".to_string());
    }
    let length = token.encode_utf16().count();
    (length < MIN_TOKEN_LENGTH)
        .then(|| format!("MCP_AUTH_TOKEN is {length} characters, and it must be at least {MIN_TOKEN_LENGTH}"))
}

/// The credential in an `Authorization: Bearer <credential>` header, or `None`.
///
/// The scheme is case-insensitive (RFC 7235) and followed by one or more spaces. Anything
/// else - another scheme, no credential, two of them - is not a bearer credential.
pub fn bearer_credential(header: Option<&str>) -> Option<&str> {
    let header = header?;
    let scheme = header.get(..6)?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let rest = &header[6..];
    let credential = rest.trim_start_matches(' ');
    if credential.len() == rest.len() || credential.is_empty() || credential.chars().any(char::is_whitespace)
    {
        return None;
    }
    Some(credential)
}

/// SHA-256 of `token`, so two values of any length compare in constant time.
#[derive(Clone)]
pub struct ExpectedToken([u8; 32]);

impl ExpectedToken {
    /// The digest of a token that already passed `token_problem`.
    pub fn new(token: &str) -> Self {
        Self(Sha256::digest(token.as_bytes()).into())
    }

    fn matches(&self, credential: &str) -> bool {
        let digest: [u8; 32] = Sha256::digest(credential.as_bytes()).into();
        bool::from(digest.ct_eq(&self.0))
    }
}

/// Middleware: let a request through only when it carries the token, except exactly
/// `GET /healthz`, which an orchestrator's probe calls without one.
///
/// A refusal is a 401 with `WWW-Authenticate: Bearer`. A request that sent a credential which
/// did not match also gets `error="invalid_token"`, so a client can tell a missing header from
/// a wrong one. Neither answer reveals anything about the token.
pub async fn require_bearer_token(
    State(expected): State<ExpectedToken>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() == axum::http::Method::GET && request.uri().path() == "/healthz" {
        return next.run(request).await;
    }

    let header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let credential = bearer_credential(header);
    if credential.is_some_and(|credential| expected.matches(credential)) {
        return next.run(request).await;
    }

    let (challenge, message) = match credential {
        None => (
            r#"Bearer realm="mcp""#,
            "Unauthorized: send the token as Authorization: Bearer <token>",
        ),
        Some(_) => (
            r#"Bearer realm="mcp", error="invalid_token""#,
            "Unauthorized: the token is not valid",
        ),
    };
    let mut response = rpc_error(StatusCode::UNAUTHORIZED, -32001, message);
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static(challenge));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_token_is_a_problem_only_when_one_is_required() {
        assert_eq!(
            token_problem("", true).as_deref(),
            Some("MCP_AUTH_TOKEN is not set")
        );
        assert_eq!(token_problem("", false), None);
    }

    #[test]
    fn a_short_token_is_refused_even_when_none_is_required() {
        let problem = token_problem("short", false).unwrap();
        assert!(
            problem.contains("is 5 characters") && problem.contains("at least 32"),
            "{problem}"
        );
        assert_eq!(token_problem(&"t".repeat(32), true), None);
    }

    #[test]
    fn only_a_bearer_header_with_one_credential_is_read() {
        assert_eq!(bearer_credential(Some("Bearer abc")), Some("abc"));
        assert_eq!(bearer_credential(Some("bearer   abc")), Some("abc"));
        assert_eq!(bearer_credential(Some("BEARER abc")), Some("abc"));
        for header in [
            "Bearer",
            "Bearer ",
            "Bearerabc",
            "Basic abc",
            "Bearer a b",
            "Bearer abc ",
            "",
        ] {
            assert_eq!(bearer_credential(Some(header)), None, "{header:?}");
        }
        assert_eq!(bearer_credential(None), None);
    }

    #[test]
    fn the_digest_compares_the_whole_token() {
        let expected = ExpectedToken::new(&"t".repeat(32));
        assert!(expected.matches(&"t".repeat(32)));
        assert!(!expected.matches(&"t".repeat(31)));
        assert!(!expected.matches(&"x".repeat(32)));
    }
}
