# Install by Git Clone

The server is installed from a clone and run with Node. It is not installed from npm.

## Requirements

Node.js 22 or newer, and git. The server is written in TypeScript: `npm install` compiles it
to `dist/` (its `prepare` step runs `tsc`), and clients run `dist/index.js`. After pulling a
new version, run `npm install` again, or `npm run build`, to recompile.

## Clone and test

PowerShell:

```powershell
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic; cd lxagents-agentic; npm install; npm test
```

bash:

```bash
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic && cd lxagents-agentic && npm install && npm test
```

## Register it with a client

### stdio (the default for one client)

```json
{
  "mcpServers": {
    "lxagents-agentic": {
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "C:/path/to/lxagents-agentic"
    }
  }
}
```

Claude Code, from the clone (PowerShell):

```powershell
claude mcp add lxagents-agentic -- node "$PWD\dist\index.js"
```

stdio needs no token and no environment variable.

### HTTP on this machine

```powershell
npm run start:http
```

This binds `127.0.0.1:3000` and needs no token; only `localhost` `Host` headers are answered.

```json
{ "mcpServers": { "lxagents-agentic": { "type": "http", "url": "http://localhost:3000/mcp" } } }
```

### HTTP as a server

Any other bind needs `MCP_AUTH_TOKEN`, at least 32 characters, and every client sends it as
`Authorization: Bearer <token>`. See [Environment](../environments/env.md).

## After registering

A client loads its servers when a session starts, so a server added mid-session stays absent
until the session restarts. Check it with `node dist/cli.js tools`, which prints the same list
the client should see.
