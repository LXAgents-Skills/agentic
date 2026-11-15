//! The HTTP transport: refuse to start without a required token, bind, serve, drain.

pub mod app;
pub mod auth;
pub mod host;
pub mod rpc_error;

use std::time::Duration;

use tokio::net::TcpListener;

use crate::{config::Config, identity::SERVER_ID, server::AgenticServer, version::VERSION};
use app::{AppOptions, router};
use auth::{MIN_TOKEN_LENGTH, configured_token, token_problem};

/// How long in-flight requests get after a shutdown signal before they are cut off.
const DRAIN_GRACE: Duration = Duration::from_secs(5);

/// `host:port` for a bind, with an IPv6 literal bracketed.
fn bind_address(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

/// Resolves on the first SIGTERM or SIGINT, and returns its name.
async fn shutdown_signal() -> &'static str {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
        tokio::select! {
            _ = term.recv() => "SIGTERM",
            _ = tokio::signal::ctrl_c() => "SIGINT",
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        "SIGINT"
    }
}

/// The startup lines: where it listens and which controls are on. None carries the token.
fn announce(config: &Config, port: u16, token_set: bool) {
    let where_ = if config.host == "0.0.0.0" {
        "all interfaces"
    } else {
        config.host.as_str()
    };
    eprintln!("{SERVER_ID} {VERSION} serving over http on :{port}/mcp ({where_})");
    eprintln!(
        "{SERVER_ID} {VERSION} {} worker thread(s), from MCP_CLUSTER_WORKERS or one per CPU.",
        config.workers.max(1)
    );
    if token_set {
        eprintln!("{SERVER_ID} {VERSION} bearer token required on every route except GET /healthz.");
    } else {
        let limit = if config.allowed_hosts.is_empty() {
            "localhost"
        } else {
            "MCP_ALLOWED_HOSTS"
        };
        eprintln!(
            "{SERVER_ID} {VERSION} bound to loopback, so no bearer token is required; Host is limited to {limit}."
        );
    }
    // Said out loud, because the default is the unguarded one.
    if config.allowed_hosts.is_empty() && token_set {
        eprintln!(
            "{SERVER_ID} {VERSION} MCP_ALLOWED_HOSTS is unset, so no Host header allow-list is applied."
        );
    }
}

/// Serve HTTP as `config` says. An `Err` is a line for stderr and exit code 1.
pub async fn serve(server: AgenticServer, config: &Config) -> Result<(), String> {
    // Fail closed, before anything binds: HTTP without a token on anything but loopback is an
    // open door on a network.
    let token = configured_token();
    if let Some(problem) = token_problem(&token, !config.local) {
        return Err(format!(
            "{SERVER_ID} {VERSION} will not start the HTTP transport: {problem}. Set MCP_AUTH_TOKEN to a secret of at least {MIN_TOKEN_LENGTH} characters, for example the output of: openssl rand -hex 32. stdio, and HTTP bound to 127.0.0.1, need no token."
        ));
    }
    let port = config.port.ok_or_else(|| {
        format!("{SERVER_ID} {VERSION} will not start the HTTP transport: PORT is not a port number.")
    })?;

    let token_set = !token.is_empty();
    let app = router(
        server,
        AppOptions {
            token,
            allowed_hosts: config.allowed_hosts.clone(),
            local: config.local,
        },
    )?;
    let listener = TcpListener::bind(bind_address(&config.host, port))
        .await
        .map_err(|error| {
            format!(
                "{SERVER_ID} {VERSION} cannot listen on {}:{port}: {error}",
                config.host
            )
        })?;
    let port = listener.local_addr().map_or(port, |address| address.port());
    announce(config, port, token_set);

    // Stop accepting on the signal, let in-flight requests finish, and cut off whatever is
    // still running after the grace period rather than waiting on it forever.
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let graceful = axum::serve(listener, app).with_graceful_shutdown(async move {
        let signal = shutdown_signal().await;
        eprintln!("{SERVER_ID} {VERSION} {signal}, draining");
        let _ = stop.send(true);
    });
    let deadline = async move {
        let mut stopped = stopped;
        let _ = stopped.wait_for(|stopped| *stopped).await;
        tokio::time::sleep(DRAIN_GRACE).await;
    };
    tokio::select! {
        result = graceful => result.map_err(|error| format!("{SERVER_ID} {VERSION} http server failed: {error}")),
        () = deadline => Ok(()),
    }
}
