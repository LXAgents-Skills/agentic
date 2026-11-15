# Docker

The image is built in two stages. The first, on `rust:1-bookworm`, compiles the binary with
`cargo build --release --locked`, with `content/` compiled into it. The second, on
`debian:bookworm-slim`, holds only that binary and runs it as a non-root user. It defaults to
HTTP on `0.0.0.0:3000`, so it needs a token.

```bash
docker build -t lxagents-agentic .
docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-agentic
```

Without `MCP_AUTH_TOKEN` the container prints why and exits 1.

stdio from the same image needs `-i`, or the server sees closed input and exits:

```bash
docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-agentic
```

Set `MCP_ALLOWED_HOSTS` whenever the container is reachable from anywhere but this machine.
`GET /healthz` answers without a token, for an orchestrator's probe.

A change to `content/` reaches the image only when the image is rebuilt.
