# Install by Git Clone

The server is installed from a clone, built with cargo, and run as one binary. It is not
published to crates.io or npm.

## Requirements

git, and Rust 1.88 or newer with cargo ([rustup](https://rustup.rs)). `cargo build --release`
compiles the server to `target/release/lxagents-agentic` (`lxagents-agentic.exe` on Windows).
`content/` is compiled into the binary, so the binary runs on its own, from anywhere.

After pulling a new version, run `cargo build --release` again. The binary keeps serving the
set it was built with until it is rebuilt.

## Clone, build and test

PowerShell:

```powershell
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic; cd lxagents-agentic; cargo build --release; cargo test
```

bash:

```bash
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic && cd lxagents-agentic && cargo build --release && cargo test
```

## Register it with a client

### stdio (the default for one client)

The binary with no arguments serves over stdio.

```json
{
  "mcpServers": {
    "lxagents-agentic": {
      "command": "C:/path/to/lxagents-agentic/target/release/lxagents-agentic.exe"
    }
  }
}
```

On Linux or macOS the command is `/path/to/lxagents-agentic/target/release/lxagents-agentic`.

Claude Code, from the clone (PowerShell):

```powershell
claude mcp add lxagents-agentic -- "$PWD\target\release\lxagents-agentic.exe"
```

bash:

```bash
claude mcp add lxagents-agentic -- "$PWD/target/release/lxagents-agentic"
```

stdio needs no token and no environment variable.

### HTTP on this machine

```bash
target/release/lxagents-agentic serve --http --host 127.0.0.1
```

This binds `127.0.0.1:3000` and needs no token; only `localhost` `Host` headers are answered.

```json
{ "mcpServers": { "lxagents-agentic": { "type": "http", "url": "http://localhost:3000/mcp" } } }
```

### HTTP as a server

Any other bind needs `MCP_AUTH_TOKEN`, at least 32 characters, and every client sends it as
`Authorization: Bearer <token>`. See [Environment](../environments/env.md).

## Coming from 0.1.1

Before 0.1.2 the server was TypeScript, run as `node dist/index.js` after `npm install`. A
client still configured that way fails to start, because `dist/` no longer exists. Build with
`cargo build --release`, and change the client's command to the binary, with no `args` and no
`cwd`.

## After registering

A client loads its servers when a session starts, so a server added mid-session stays absent
until the session restarts. Check it with `target/release/lxagents-agentic tools`, which
prints the same list the client should see.
