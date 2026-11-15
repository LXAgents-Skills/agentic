# Architecture

## Source and build

The server is Rust, in one crate: a library in `src/lib.rs` and a binary in `src/main.rs`. It
is built on the official MCP Rust SDK, [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk),
for the protocol and both transports, with [`axum`](https://github.com/tokio-rs/axum) for the
HTTP routes and middleware and `tokio` as the runtime. `cargo build --release` compiles it to
`target/release/lxagents-agentic`, which is what clients and the Docker image run.

`build.rs` embeds every `.md` file in `content/` into the binary, so the binary carries its set
with it and serves nothing from disk. Cargo re-runs `build.rs` whenever `content/` changes.

## From a request to a file

1. `src/main.rs` parses the arguments (`src/cli.rs`), builds and checks the set, reads the
   environment once (`src/config.rs`), and starts a runtime sized for the transport.
2. stdio runs `src/stdio.rs`. HTTP runs `src/http/mod.rs`, which refuses to start without a
   required token, then binds and serves the router from `src/http/app.rs`.
3. Each stdio connection, and each HTTP request, is answered by an `AgenticServer` from
   `src/server.rs`: `tools/list` returns one tool per entry of `instructions()`, and
   `tools/call` returns that entry's file as one text block.
4. `instructions()` is built once, by `src/instructions/surface.rs`, from the table `build.rs`
   wrote: one tool per `.md` file, its ID from `src/instructions/naming.rs`, its description
   from the file's frontmatter, and the file's text byte for byte.

## Why the set is checked at boot

A file without a description, an ID over 64 characters, two files with one ID, or an override
naming a missing file all stop the process with a message naming the file and the fix. A set
that cannot be routed on should not start.

## One name, one place

`src/identity.rs` holds the org, repo, server ID, title and summary. The tool prefix, every ID,
the health check and every log line come from it. `Cargo.toml` holds the version.

## HTTP

`POST /mcp` is stateless: rmcp's `StreamableHttpService` with sessions off, answering each
request with a fresh server. `GET /healthz` is the only route without a token. The `Host`
allow-list, then the token check, then the JSON body check (4 MB at most) run before the routes.
`MCP_CLUSTER_WORKERS` sets the number of runtime threads; nothing is forked. See
[Environment](../environments/env.md).
