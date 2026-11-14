---
name: memory-tasks-rust
description: Migrating LXAgents-Skills/agentic from TypeScript to Rust on the official MCP Rust SDK (rmcp) with axum for HTTP, keeping the tool surface and the HTTP security model.
---

# Task: migrate agentic to Rust

**Status: in progress.**

The owner asked for every repository in the organization to move from TypeScript to Rust,
using the official MCP Rust SDK (`rmcp`, from `modelcontextprotocol/rust-sdk`) and `axum` as
the web framework, with one thread and one plan per repository. This is the agentic one. The
owner is away, so choices were made autonomously with the recommended option, and merging
without human review is allowed.

## What changes

- The server is Rust: `Cargo.toml`, `build.rs`, `src/**/*.rs`, `tests/*.rs`. `rmcp` serves stdio
  and stateless Streamable HTTP; `axum` carries the HTTP routes and middleware; `tokio` runs it.
- One binary, `lxagents-agentic`. With no arguments it serves as the environment says, as
  `node dist/index.js` did; `serve`, `tools`, `--version` and `--help` are the old CLI.
- `content/` is embedded at compile time by `build.rs` and validated at boot.
- `MCP_CLUSTER_WORKERS` sets tokio worker threads instead of forked processes.
- `package.json`, the lockfile, `tsconfig*.json`, `src/**/*.ts`, `test/` and `scripts/init` are
  removed.
- The Dockerfile builds with `rust` and runs the binary on `debian:bookworm-slim`.
- Docs and rules give the new commands: `cargo build --release`, then
  `target/release/lxagents-agentic`.
- Release `0.1.2`, a patch bump, the owner's default.

## What does not change

- The tool surface: the six IDs, their descriptions, the bytes served.
- The `{org}_{repo}_{base}` ID rule, the 64-character cap and `OVERRIDE_BASES`.
- `content/`, and the rule that work state is written outside the repository.
- HTTP security: token only off loopback, at least 32 characters, exact `GET /healthz`
  exemption, the `Host` allow-list (localhost when tokenless), the same JSON-RPC refusals.
- Installing by git clone.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record | `plan/rust` | |
| 2 | Rust server, tests, Dockerfile; TypeScript removed | `refactor/rust` | |
| 3 | README, wiki, rules | `docs/rust` | |
| 4 | Version 0.1.2, changelog, close this record | `release/0.1.2` | |

## Done criteria

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and
`cargo build --release` exit 0, and a stdio client gets identical results for every tool from
the TypeScript build at `9c38975` and from the Rust binary.

## Decisions taken without the owner

- **Branch names follow the shared convention**, not the harness's default branch name.
- **One binary; no arguments means serve.** A client config needs only the binary path.
- **`content/` is embedded at compile time**, so the binary is self-contained; cargo rebuilds
  when `content/` changes.
- **`MCP_CLUSTER_WORKERS` is a thread count.** `1` runs a single-threaded runtime.
- **`scripts/init` is removed, not ported.** It renames a template copy; this repository is
  already initialized, and the template keeps its own.
- **No CI workflow is added**; none exists today.
- **HTTP answers stay server-sent events**, the stateless default of the TypeScript SDK.

## Progress

### Task 1 — plan/rust

Landed: this record and its row in `memory-index.md`.
