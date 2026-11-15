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

### Task 2 — refactor/rust

Landed. The TypeScript server, its tests, `scripts/init`, `package.json`, the lockfile and both
`tsconfig` files are removed. In their place:

- `Cargo.toml` (edition 2024, `rust-version` 1.88) and the committed `Cargo.lock`. Runtime
  crates: `rmcp` 3.10 (`server`, `transport-io`, `transport-streamable-http-server`), `axum`
  0.8, `tokio` 1, `serde_json` (key order preserved), `sha2` 0.11 and `subtle` 2. Test-only:
  `rmcp` with `client` and `transport-child-process`, and `regex`.
- `build.rs` writes the table of embedded `content/` files; `src/instructions/surface.rs`
  builds and checks it at boot with the same refusals as before.
- `src/lib.rs` holds every module, so the integration tests drive the server in-process;
  `src/main.rs` is the only effectful entry point. One module per old TypeScript file, except
  `http/primary.ts` and `http/worker.ts`, which became `http/mod.rs` (no process forking), and
  the new `http/host.rs`, which replaces the SDK's `Host` middleware.
- `rustfmt.toml` sets a 110-column width.
- The Dockerfile builds with `rust:1-bookworm` and runs the binary on `debian:bookworm-slim`
  as a non-root user.

Verification:

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- `cargo test` passes 59 tests: 29 unit tests, plus 30 integration tests in `tests/surface.rs`,
  `tests/agentic_content.rs`, `tests/http.rs` and `tests/binary.rs`.
- A stdio client called every tool on the TypeScript build at `9c38975` and on the release
  binary. `initialize` (name, title, instructions), `tools/list` (names, descriptions, no
  arguments) and every `tools/call` result were identical. `tools` prints the same six lines.
- The Docker image was built and run. It refuses to start HTTP without a token (exit 1),
  answers `/healthz` with a token, and answers stdio with `-i`. The image is 123 MB. In this
  sandbox, the build needed the proxy's CA certificate added to a throwaway copy of the
  Dockerfile; the committed Dockerfile needs nothing extra.

### Task 3 — docs/rust

Landed. Every page that named a TypeScript file, an npm script or `node dist/...` now names
the Rust file or the cargo command:

- `README.md` and `wiki/guides/install-by-git-clone.md`: clone, then `cargo build --release`.
  Clients run `target/release/lxagents-agentic` with no arguments. The install guide has a
  *Coming from 0.1.1* section.
- `AGENTS.md`: the Rust stack, `tests/`, and `Cargo.toml` as the version carrier.
  `scripts/` is gone.
- `.agents/rules/repository.md` is rewritten for cargo: where things go, the commands, and a
  *Rust, one crate, one binary* section. *What must not be introduced* now forbids a build
  step other than cargo and a committed `target/`.
- `tool-ids.md`, `tool-authoring.md`, `automation-hub.md` and `http-auth.md` name the `.rs`
  files and the Rust test files. The `OVERRIDE_BASES` example is Rust.
- `.agents/wiki/context/repository-map.md` is rewritten for the Rust files.
- `wiki/information/{architecture,overview}.md`, `wiki/environments/{docker,env}.md` and
  `wiki/guides/add-a-tool.md` describe the build, the embedded set, and threads rather than
  forked workers.

`content/` is unchanged. The self corrector's generic advice to look in `package.json` is about
any repository, not this one. The released 0.1.0 and 0.1.1 changelogs and the closed task
records keep their history.
