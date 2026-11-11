---
name: memory-tasks-typescript
description: Migrating LXAgents-Skills/agentic from JavaScript to strict TypeScript compiled to dist/, with every dependency at its latest version and a clean npm audit.
---

# Task: migrate agentic to TypeScript

**Status: in progress.**

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
| 1 | This record | `plan/typescript` | |
| 2 | Latest dependencies, TS tooling, audit | `build/dependencies` | |
| 3 | tsconfig, conversion, package.json | `refactor/typescript` | |
| 4 | README, wiki, Dockerfile, rules | `docs/typescript` | |
| 5 | Version 0.1.1, changelog, close this record | `release/0.1.1` | |

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
