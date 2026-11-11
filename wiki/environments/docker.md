# Docker

The image is built in two stages on `node:22-alpine`. The first installs every dependency and
compiles `src/` with `tsc`. The second holds only the production dependencies from the
lockfile, the compiled `dist/` and `content/`, and runs `dist/index.js` as the `node` user. It defaults to HTTP on `0.0.0.0:3000`, so it needs a token.

```bash
docker build -t lxagents-agentic .
docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-agentic
```

stdio from the same image needs `-i`, or the server sees closed input and exits:

```bash
docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-agentic
```

Set `MCP_ALLOWED_HOSTS` whenever the container is reachable from anywhere but this machine.
`GET /healthz` answers without a token, for an orchestrator's probe.
