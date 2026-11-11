# 0.1.1

**Released 2026-11-11.** The server, its tests and its init script are now strict TypeScript,
compiled by `tsc` to `dist/`. Every dependency is at its latest version, and `npm audit` is
clean. The tool surface has not changed: the same IDs and descriptions, and each tool serves
its file byte for byte.

## Added

- `tsconfig.json`: ES2022, NodeNext, `strict`, `src/` → `dist/`.
- `tsconfig.check.json`: type-checks `src/`, `test/` and `scripts/` without output.
- Scripts:
  - `build` (`tsc`);
  - `typecheck`;
  - `prepare`, which builds on `npm install`;
  - `pretest`, which builds and type-checks.
- `test/dist.test.ts`: starts the compiled server with plain `node`, and checks the CLI, a
  stdio handshake, and HTTP `/healthz` on loopback.
- Dev dependencies: `typescript`, `@types/node`, `tsx`, `@types/express`.

## Changed

- Every `.js` file in `src/`, `test/` and `scripts/` is now `.ts`, typed, with the logic
  unchanged.
- `main`, `bin`, `start`, `start:http`, `tools`, `cli` and `inspect` run `dist/`. `test` runs
  `node --import tsx --test test/*.test.ts`, and `init` runs `tsx scripts/init.ts`.
- Dependency versions: `@modelcontextprotocol/sdk` ^1.32.2, `express` ^5.3.0, `zod` ^4.6.7.
- `engines.node` is now `>=22`, because Node 20 is past end of life.
- The Dockerfile has two stages, and the image runs `dist/index.js`.
- The rule against a build step now allows exactly one build step, `tsc`, and forbids
  committing `dist/`.

**What a reader has to do:**
1. Use Node 22 or later.
2. After pulling, run `npm install`, which compiles the server.
3. Change the client's server path from `src/index.js` to `dist/index.js`.

A client still pointing at `src/index.js` will fail to start, because that file no longer
exists.
