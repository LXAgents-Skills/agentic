---
name: repository-map
description: Every source file in this repository and the one thing it is responsible for, plus how a request flows through them.
---

# Repository Map

## Flow

```
src/main.rs ─ cli::parse() ─ surface::instructions() ─ load_config() ─ runtime
                                                                ├─ stdio → src/stdio.rs ─ AgenticServer
                                                                └─ http  → src/http/mod.rs ─ app::router() ─ StreamableHttpService ─ AgenticServer

AgenticServer (src/server.rs) ─ list_tools / call_tool over instructions()
instructions() (src/instructions/surface.rs) ← CONTENT_FILES, embedded by build.rs from content/**/*.md
```

Dependencies point one way: `main → cli, config → stdio, http → server → instructions → identity`.
Everything but `main.rs` is the library in `src/lib.rs`, so the tests drive it in-process.

## Files

| File | Responsible for |
|---|---|
| `Cargo.toml`, `Cargo.lock` | The crate, its version (the only place it is written) and its locked dependencies. |
| `build.rs` | Embedding `content/`: writes the `(path, text)` table to `$OUT_DIR/content_files.rs`. |
| `src/main.rs` | The entry point and the only effects: argv, the environment, the runtime, stdout for CLI output, the exit code. |
| `src/lib.rs` | Declaring the modules. |
| `src/identity.rs` | Naming the server: org, repo, server ID, title, summary; derives `tool_prefix()` and `tool_id()`. |
| `src/version.rs` | `VERSION`, from `Cargo.toml` at compile time. |
| `src/config.rs` | Reading every environment variable into one `Config`; `is_loopback()`. |
| `src/cli.rs` | Parsing argv into a `Command`; the help text. |
| `src/instructions/frontmatter.rs` | Parsing single-line frontmatter fields. |
| `src/instructions/naming.rs` | The tool ID rule: base derivation, `OVERRIDE_BASES`, the 64-character cap. |
| `src/instructions/surface.rs` | Building the tool list from the embedded set and refusing a malformed one. |
| `src/server.rs` | The rmcp `ServerHandler`: `initialize` info, `tools/list`, `tools/call`. |
| `src/stdio.rs` | The stdio transport. |
| `src/http/mod.rs` | Refusing to start without a required token; binding, the startup lines, draining on shutdown. |
| `src/http/app.rs` | The axum router: `Host` check, token check, JSON body check, `/mcp`, `/healthz`, 404 and 405. |
| `src/http/auth.rs` | The bearer token: reading it, judging it, checking a request. |
| `src/http/host.rs` | The `Host` allow-list. |
| `src/http/rpc_error.rs` | The JSON-RPC error envelope. |
| `tests/common/mod.rs` | Test helpers: in-memory client, raw HTTP requests, spawning the binary. |
| `tests/*.rs` | Integration tests: the surface, the agentic content rules, HTTP, the binary. |
| `target/` | Build output, git-ignored; `target/release/lxagents-agentic` is what clients and the image run. |
