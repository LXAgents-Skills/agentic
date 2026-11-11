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
| Org, repo, server ID, title, summary | `src/identity.ts` — the only file that names the server |
| The ID rule and `OVERRIDE_BASES` | `src/instructions/naming.ts` |
| The surface, derived from `content/` | `src/instructions/from-content.ts` |
| `registerTool` and `listTools()` | `src/server.ts` |
| Every environment variable | `src/config.ts` (except `MCP_AUTH_TOKEN`, read in `src/http/auth.ts`) |
| Entry point and transports | `src/index.ts` → `src/run.ts` → `src/stdio.ts` or `src/http/` |
| The command line | `src/cli.ts` |
| The rename script | `scripts/init.ts`, with its pure parts in `scripts/init/` |
| Tests | `test/{subject}.test.ts` |

## Commands

```bash
npm install          # Node 22+; `prepare` compiles src/ to dist/
npm run build        # tsc: src/*.ts -> dist/*.js
npm run typecheck    # tsc over src/, test/ and scripts/, no output
npm test             # build, typecheck, then node --import tsx --test test/*.test.ts
npm run tools        # every tool ID, its length, and its description
npm start            # stdio
npm run start:http   # HTTP on 127.0.0.1:3000, no token needed
npm run inspect      # MCP Inspector against the stdio server
npm run init -- --org <org> --repo <repo>   # once, in a new repository made from the template
```

## Nothing writes to stdout except the CLI

On stdio, stdout is the JSON-RPC channel. Server-side logging goes to stderr; only CLI
commands write to stdout.

## Installed by git clone

People install this server by cloning it, running `npm install` (whose `prepare` step builds
`dist/`), and running `node dist/index.js`, not from npm or `npx`. `package.json` is `private`, so it cannot be published by accident. Docs give the clone
path first.

## TypeScript, compiled to `dist/`

The source is strict TypeScript in `src/`. `tsc` compiles it to `dist/`, and only `dist/` runs
in production: `main`, `bin`, `start` and the Docker image all point there. `dist/` is
git-ignored and is never committed.

* Imports use `.js` specifiers (`./config.js`), which NodeNext resolves to the `.ts` source.
* No `any`: `strict` is on and `npm run typecheck` covers `src/`, `test/` and `scripts/`.
* Tests and `scripts/init.ts` run through `tsx`; the tests in `test/dist.test.ts` start the
  compiled `dist/` with plain `node`, so a build that breaks the shipped server fails the suite.
* `tsc` is the only build step. No bundler, minifier or second compiler.

## What must not be introduced

* A build step other than `tsc`, or a committed `dist/`.
* A second list of tools, or a tool registered outside `src/server.ts`.
* A tool that takes an argument, a verb, a credential, or reaches a network.
* A source file other than `src/identity.ts` that spells the org, repo or server ID.
* A copy of the shared instruction set. Read it through its connector.
* A change to the `@modelcontextprotocol/sdk` calls (`registerTool`, `McpServer`, the
  transports) made for style alone.
