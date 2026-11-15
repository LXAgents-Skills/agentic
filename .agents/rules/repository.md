---
name: repository-rules
description: Rules for this repository - where things go, the commands, the stdout ban, install by git clone, and what must not be introduced.
---

# Repository Rules

## Where things go

| Thing | Path |
|---|---|
| The served set | `content/` |
| The hub over it | `content/automation.md` |
| Embedding the set in the binary | `build.rs` |
| Org, repo, server ID, title, summary | `src/identity.rs` — the only file that names the server |
| The version | `Cargo.toml` — the only place it is written |
| The ID rule and `OVERRIDE_BASES` | `src/instructions/naming.rs` |
| The surface, derived from `content/` | `src/instructions/surface.rs` |
| `tools/list`, `tools/call`, `initialize` info | `src/server.rs` |
| Every environment variable | `src/config.rs` (except `MCP_AUTH_TOKEN`, read in `src/http/auth.rs`) |
| Entry point and the only effects | `src/main.rs` → `src/stdio.rs` or `src/http/` |
| The command line | `src/cli.rs` |
| Tests | unit tests beside the code; integration tests in `tests/{subject}.rs` |

## Commands

```bash
cargo build --release                      # target/release/lxagents-agentic, content/ embedded
cargo test                                 # unit and integration tests
cargo fmt --check                          # formatting
cargo clippy --all-targets -- -D warnings  # lints; a warning fails
cargo run --quiet -- tools                 # every tool ID, its length, and its description
cargo run --quiet                          # stdio
cargo run --quiet -- serve --http --host 127.0.0.1   # HTTP on 127.0.0.1:3000, no token needed
npx @modelcontextprotocol/inspector target/release/lxagents-agentic   # MCP Inspector against stdio
```

A task is done when the first four exit 0.

## Nothing writes to stdout except the CLI

On stdio, stdout is the JSON-RPC channel. Server-side logging goes to stderr; only the `tools`,
`--help` and `--version` commands in `src/main.rs` write to stdout.

## Installed by git clone

People install this server by cloning it, running `cargo build --release`, and pointing a
client at `target/release/lxagents-agentic`, not from crates.io. `Cargo.toml` sets
`publish = false`, so it cannot be published by accident. Docs give the clone path first.

## Rust, one crate, one binary

* The library in `src/lib.rs` holds every module; `src/main.rs` is the binary and the only
  place that reads argv or the environment, starts a runtime, or picks an exit code.
* The MCP protocol and both transports come from `rmcp`; HTTP routing and middleware from
  `axum`. No second MCP or HTTP framework.
* `content/` is embedded by `build.rs`, so the binary serves nothing from disk. A content
  change ships only in a rebuilt binary.
* No `unsafe`. `cargo clippy --all-targets -- -D warnings` must stay clean.
* `Cargo.lock` is committed, and the Docker image builds with `--locked`.

## What must not be introduced

* A build step other than cargo (`build.rs` included), or a committed `target/`.
* A second list of tools, or a tool served anywhere but from `instructions()` in `src/server.rs`.
* A tool that takes an argument, a verb, a credential, or reaches a network.
* A source file other than `src/identity.rs` that spells the org, repo or server ID.
* A copy of the shared instruction set. Read it through its connector.
* A change to the `rmcp` calls (`ServerHandler`, `serve`, `StreamableHttpService`) made for
  style alone.
