# 0.1.2

**Released 2026-11-15.** The server is now Rust: one binary, `lxagents-agentic`, built on the
official MCP Rust SDK (`rmcp`) with `axum` for HTTP. The tool surface has not changed: the
same six IDs and descriptions, and every tool serves its file byte for byte.

## Added

- `Cargo.toml`, `Cargo.lock` and `build.rs`. `build.rs` compiles `content/` into the binary.
- `src/**/*.rs`: a library holding every module, and a binary that is the only effectful entry
  point.
- `tests/*.rs`: 30 integration tests, against the library and the built binary, and 29 unit
  tests beside the code. Together they cover what the TypeScript suite covered.
- `rustfmt.toml`.
- `.agents/rules/repository.md` has a *Rust, one crate, one binary* section.

## Changed

- **The run command.** Build with `cargo build --release`, and run
  `target/release/lxagents-agentic` with no arguments for stdio. The CLI's `serve`, `tools`,
  `--help` and `--version` are subcommands of the same binary.
- `MCP_CLUSTER_WORKERS` sets the number of HTTP worker threads; nothing is forked. `0` or `1`
  runs one thread.
- `content/` is embedded at build time, so a change to it reaches a client only after a
  rebuild.
- The Dockerfile builds on `rust:1-bookworm` and runs the binary on `debian:bookworm-slim` as a
  non-root user.
- Every rule and wiki page names the Rust files and the cargo commands.

The `Host` allow-list, the bearer token (required off loopback only, at least 32 characters,
exact `GET /healthz` exemption) and every JSON-RPC refusal behave as before.

## Removed

- The TypeScript sources and tests, `package.json`, `package-lock.json`, `tsconfig.json`,
  `tsconfig.check.json` and `dist/`.
- `scripts/init`, the template rename script. This repository is already initialized, and
  the template keeps its own.

**What a reader has to do:**

1. Install Rust 1.88 or newer.
2. After pulling, run `cargo build --release`.
3. Change the client's server from `node dist/index.js` (with `cwd`) to the binary
   `target/release/lxagents-agentic`, with no arguments.

A client still pointing at `dist/index.js` will fail to start, because that file no longer
exists.
