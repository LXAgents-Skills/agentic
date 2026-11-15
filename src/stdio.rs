//! Serve over stdio.
//!
//! stdout is the JSON-RPC channel here, so nothing on this path writes to it, and nothing on
//! it reads `MCP_AUTH_TOKEN`: whoever can use the pipe already runs the process.

use rmcp::{ServiceExt, transport::stdio};

use crate::{identity::SERVER_ID, server::AgenticServer, version::VERSION};

/// Serve `server` on stdin and stdout until the client closes the pipe.
pub async fn serve(server: AgenticServer) -> Result<(), String> {
    let running = server
        .serve(stdio())
        .await
        .map_err(|error| format!("stdio handshake failed: {error}"))?;
    eprintln!("{SERVER_ID} {VERSION} serving over stdio");
    running
        .waiting()
        .await
        .map(|_| ())
        .map_err(|error| format!("stdio server stopped: {error}"))
}
