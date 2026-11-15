//! The HTTP transport, as an application.
//!
//! This module builds an axum `Router` and returns it. **It does not listen** - `http::serve`
//! owns the port, so the app can be reasoned about and tested without binding one.
//!
//! The surface is deliberately narrow: `POST /mcp` and `GET /healthz`, a JSON-RPC 404 for
//! everything else, and a 405 for any other method on `/mcp`. Requests pass, in order: the
//! `Host` allow-list, then the bearer token (everything except exactly `GET /healthz`), then the
//! JSON body check, then the routes.

use std::sync::Arc;

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post_service},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};
use serde_json::json;

use super::{
    auth::{ExpectedToken, require_bearer_token, token_problem},
    host::{AllowedHosts, LOCALHOST_HOSTS, require_allowed_host},
    rpc_error::rpc_error,
};
use crate::{identity::SERVER_ID, server::AgenticServer, version::VERSION};

/// The request body ceiling, in bytes.
pub const BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

/// What `router` takes.
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    /// The bearer token every request but `GET /healthz` must carry; empty for none.
    pub token: String,
    /// The `Host` allow-list; empty applies no check, unless the server is tokenless.
    pub allowed_hosts: Vec<String>,
    /// Set when the server binds loopback. Only then may `token` be empty. It defaults to
    /// `false`, so leaving it out can never turn the token check off.
    pub local: bool,
}

/// Whether a request declares a JSON body.
fn is_json(request: &Request) -> bool {
    request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|mime| {
            let mime = mime.trim().to_ascii_lowercase();
            mime == "application/json" || mime.ends_with("+json")
        })
        .unwrap_or(false)
}

/// Middleware: read a JSON body of at most `BODY_LIMIT_BYTES` and check that it parses.
///
/// An oversized body and a malformed one get the same 400 / `-32700`, as they always have.
/// It runs after the token check, so a caller without the token cannot make the server read
/// up to 4 MB.
async fn check_json_body(request: Request, next: Next) -> Response {
    if !is_json(&request) {
        return next.run(request).await;
    }
    let (parts, body) = request.into_parts();
    let parsed = to_bytes(body, BODY_LIMIT_BYTES).await.ok().filter(|bytes| {
        bytes.iter().all(u8::is_ascii_whitespace)
            || serde_json::from_slice::<serde_json::Value>(bytes).is_ok()
    });
    match parsed {
        Some(bytes) => next.run(Request::from_parts(parts, Body::from(bytes))).await,
        None => rpc_error(
            StatusCode::BAD_REQUEST,
            -32700,
            "Parse error: request body is not valid JSON",
        ),
    }
}

/// The health check. Answers without a session, a request, or a tool.
async fn healthz() -> Response {
    Json(json!({ "status": "ok", "server": SERVER_ID, "version": VERSION })).into_response()
}

/// Any other method on `/mcp`: the path exists, so it is a 405 and not a 404.
async fn method_not_allowed(request: Request) -> Response {
    rpc_error(
        StatusCode::METHOD_NOT_ALLOWED,
        -32000,
        format!("{} is not supported in stateless mode", request.method()),
    )
}

/// Anything else is not a route this server has.
async fn not_found(request: Request) -> Response {
    let target = request
        .uri()
        .path_and_query()
        .map_or_else(|| request.uri().path().to_string(), ToString::to_string);
    rpc_error(StatusCode::NOT_FOUND, -32601, format!("Not found: {target}"))
}

/// The Skill MCP endpoint: stateless, a fresh server per request, no sessions to keep bounded.
fn mcp_service(server: AgenticServer) -> StreamableHttpService<AgenticServer, NeverSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_max_request_body_bytes(BODY_LIMIT_BYTES)
        // The allow-list runs in `require_allowed_host`, ahead of every route.
        .disable_allowed_hosts();
    StreamableHttpService::new(move || Ok(server.clone()), Arc::default(), config)
}

/// Build the HTTP application.
///
/// **There is no way to build it without a token off loopback**: a missing or too-short token
/// is an error, so an unauthenticated HTTP server cannot come from forgetting an option.
pub fn router(server: AgenticServer, options: AppOptions) -> Result<Router, String> {
    if let Some(problem) = token_problem(&options.token, !options.local) {
        return Err(format!("Refusing to build the HTTP app: {problem}."));
    }

    let open = options.token.is_empty();
    let mut app = Router::new()
        .route("/healthz", get(healthz).fallback(not_found))
        .route(
            "/mcp",
            post_service(mcp_service(server)).fallback(method_not_allowed),
        )
        .fallback(not_found)
        .layer(middleware::from_fn(check_json_body));

    if !open {
        app = app.layer(middleware::from_fn_with_state(
            ExpectedToken::new(&options.token),
            require_bearer_token,
        ));
    }

    let hosts = if !options.allowed_hosts.is_empty() {
        Some(options.allowed_hosts)
    } else if open {
        Some(LOCALHOST_HOSTS.iter().map(ToString::to_string).collect())
    } else {
        None
    };
    if let Some(hosts) = hosts {
        app = app.layer(middleware::from_fn_with_state(
            AllowedHosts(Arc::new(hosts)),
            require_allowed_host,
        ));
    }

    Ok(app)
}
