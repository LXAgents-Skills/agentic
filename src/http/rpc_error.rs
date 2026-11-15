//! A JSON-RPC error response.
//!
//! The shape every refusal on this server uses: the same envelope a client parses for a
//! successful response, so a client never has to branch on content type to find out it was
//! refused. The 401, 403, 404, 405 and the parse errors all come through here.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// A `status` response carrying `{ jsonrpc, error: { code, message }, id: null }`.
pub fn rpc_error(status: StatusCode, code: i64, message: impl Into<String>) -> Response {
    let body = json!({ "jsonrpc": "2.0", "error": { "code": code, "message": message.into() }, "id": null });
    (status, Json(body)).into_response()
}
