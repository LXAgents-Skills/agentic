---
name: memory-tasks-typescript
description: Migrating LXAgents-Skills/agentic from JavaScript to strict TypeScript compiled to dist/, with every dependency at its latest version and a clean npm audit.
---

# Task: migrate agentic to TypeScript

**Status: done.**

The owner asked for every Node.js repository in the organization to move to TypeScript,
with one thread per repository. This is the agentic one. It follows the owner's checklist:
latest dependencies and audit, `tsconfig.json`, conversion, `package.json`, then build, test
and audit.

## What changes

- Dependencies at their latest versions. Dev dependencies added: `typescript`,
  `@types/node`, `tsx` and `@types/express`.
- `tsconfig.json`: `target` ES2022, `module` and `moduleResolution` NodeNext, `rootDir`
  `./src`, `outDir` `./dist`, `strict`.
- Every `.js` in `src/`, `test/` and `scripts/` becomes `.ts`, typed, with the logic
  unchanged.
- `package.json`:
  - `main`, `bin` and `start` point to `dist/`;
  - `build` is `tsc`;
  - `prepare` builds on `npm install`;
  - `test` is `node --import tsx --test test/*.test.ts`.
- Docs, the Dockerfile and the repository rules describe the build.
- Release `0.1.1`, a patch bump, which is the owner's default.

## What does not change

- The tool surface: the IDs, the descriptions, the bytes served.
- The transports and HTTP auth (token only off loopback).
- The `content/` set.
- Installing by git clone: clone, then `npm install`.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record | `plan/typescript` | #6 |
| 2 | Latest dependencies, TS tooling, audit | `build/dependencies` | #7 |
| 3 | tsconfig, conversion, package.json | `refactor/typescript` | #8 |
| 4 | README, wiki, Dockerfile, rules | `docs/typescript` | #9 |
| 5 | Version 0.1.1, changelog, close this record | `release/0.1.1` | #10 |

## Rules set aside, by the owner's instruction

- **"No build step"** (`AGENTS.md` and `.agents/rules/repository.md`). TypeScript compiled to
  `dist/` is a build step, so the rule is rewritten in task 4.

## Decisions taken without the owner

- **`prepare` runs `tsc`.** A git-clone install still needs only `npm install`.
- **`scripts/init` is converted as well, and runs through `tsx`.** It imports
  `src/identity`, which no longer exists as JavaScript.
- **Tests run against `src/` through `tsx`.** The process-spawning tests (HTTP, init) start
  the TypeScript entry points the same way.

## Progress

### Task 1 — plan/typescript

Landed: this record and its row in `memory-index.md`.

### Task 2 — build/dependencies

Landed. The runtime dependencies are at their latest versions: `@modelcontextprotocol/sdk`
^1.32.2, `express` ^5.3.0 and `zod` ^4.6.7. The dev dependencies are `typescript` ^7.0.2,
`@types/node` ^26.6.6, `tsx` ^4.23.15 and `@types/express` ^5.0.6. `npm audit` found 0
vulnerabilities and `npm outdated` is empty. The JavaScript suite still passes, 43 of 43.

### Task 3 — refactor/typescript

Landed. Every `.js` in `src/`, `test/` and `scripts/` was renamed to `.ts` with `git mv`, then
typed under `strict` with the logic unchanged. The types added:

- `Config` (config);
- `ToolConfig`, `ContentInstruction` and `ContentSurface` (the surface);
- `AppOptions` (HTTP app);
- `Flags`, `IdentityFlags` and `Identity` (init);
- the SDK's `CallToolResult` for the tool handlers;
- express `RequestHandler`, `ErrorRequestHandler`, `Response` and `Express`.

Imports keep their `.js` specifiers, as NodeNext resolves them.

Configuration:

- `tsconfig.json` builds `src/` to `dist/` with ES2022, NodeNext, strict and source maps.
- `tsconfig.check.json` type-checks `src/`, `test/` and `scripts/` without emitting.

`package.json`:

- `main` and `bin` point to `dist/`, and `files` ships `dist/`.
- Scripts: `build` (`tsc`), `typecheck`, `prepare` (build on install), and `pretest` (build
  and typecheck).
- `test` is `node --import tsx --test test/*.test.ts`. `start`, `start:http`, `cli`, `tools`
  and `inspect` run `dist/`, and `init` runs `tsx scripts/init.ts`.
- `engines` is now `>=22`. Node 20 reached end of life in April 2026, and `--test` globs need
  Node 21 or later.

`dist/` is git-ignored.

Changes to logic that the move forced:

- `derivedBase` falls back to the whole path when `split` yields nothing, which cannot happen.
- `bearerCredential` uses `match?.[1] ?? null`.
- `init` rewrites `src/identity.ts`, sets new bins to `dist/`, and boots `src/cli.ts`
  through `tsx`.

The process-spawning tests start the TypeScript entry points with `--import tsx`.

The new `test/dist.test.ts` starts the compiled `dist/` with plain `node`. It checks the CLI
version and tool list, a stdio handshake that lists the tools, and HTTP `/healthz` on
loopback with no token.

The Dockerfile is two stages: the first runs `tsc`, and the image runs `dist/index.js`.
Docker itself is not available here, so the image's steps were reproduced by hand: `npm ci`
and a build, then `npm ci --omit=dev` with `dist/` and `content/` copied in, and `/healthz`
answered.

Results: 46 tests pass. Before the move it was 42 tests; the old count of 43 included
`helpers.js`, which node ran as a file. `npm run typecheck` is clean and `npm audit` found 0
vulnerabilities.

### Task 4 — docs/typescript

Landed. Every page that named a `src/`, `test/` or `scripts/` `.js` file now names the `.ts`
file. Commands that run the server now run `dist/`: `node dist/index.js`, `node dist/cli.js`,
and the client config `args: ["dist/index.js"]`.

- `AGENTS.md` and the install guide say Node 22+, strict TypeScript compiled by `tsc`, and that
  `npm install` builds.
- The architecture page has a *Source and build* section.
- The Docker page describes the two stages.
- The repository map lists `tsconfig.json`, `tsconfig.check.json` and `dist/`.
- In `.agents/rules/repository.md`:
  - "A build step" under *What must not be introduced* becomes "a build step other than
    `tsc`, or a committed `dist/`". This is the owner's instruction, set aside as recorded
    above.
  - A new *TypeScript, compiled to `dist/`* section covers `.js` import specifiers, no `any`,
    `tsx` for tests and `init`, and the `dist/` tests.
  - The command block lists `build` and `typecheck`.

46 tests pass.

### Task 5 — release/0.1.1

Landed. `package.json` and `package-lock.json` are at `0.1.1`.
`wiki/logs/0/1/1/CHANGELOG.md` is written, with the client path change called out, and has its
row in `logs-index.md`. The `PR` column is filled in (#6 to #10) and this record is closed.
