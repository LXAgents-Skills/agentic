import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { once } from "node:events";
import { createServer as createNetServer, type AddressInfo } from "node:net";
import { join } from "node:path";
import { test } from "node:test";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { ROOT } from "../src/paths.js";
import { listTools } from "../src/server.js";
import { version } from "../src/version.js";

/*
 * The compiled server, as a client installs it: `npm run build` (run by `pretest` and by
 * `prepare`) writes `dist/`, and these tests start `dist/` with plain `node`, no `tsx`.
 */

const DIST_INDEX = join(ROOT, "dist/index.js");
const DIST_CLI = join(ROOT, "dist/cli.js");

test("the build wrote both entry points", () => {
  assert.ok(existsSync(DIST_INDEX), "run npm run build first");
  assert.ok(existsSync(DIST_CLI), "run npm run build first");
});

test("the compiled CLI reports the package version and the same tool list", () => {
  assert.equal(spawnSync(process.execPath, [DIST_CLI, "--version"], { encoding: "utf8" }).stdout.trim(), version);

  const tools = spawnSync(process.execPath, [DIST_CLI, "tools"], { encoding: "utf8" });
  assert.equal(tools.status, 0, tools.stderr);
  const names = tools.stdout.trim().split("\n").map((line) => line.split(/\s+/)[0]);
  assert.deepEqual(names, listTools().map((tool) => tool.name));
});

test("the compiled server answers over stdio", async () => {
  const client = new Client({ name: "dist-test", version: "0.0.0" });
  await client.connect(new StdioClientTransport({ command: process.execPath, args: [DIST_INDEX], stderr: "ignore" }));
  try {
    const { tools } = await client.listTools();
    assert.deepEqual(tools.map((tool) => tool.name).sort(), listTools().map((tool) => tool.name).sort());
  } finally {
    await client.close();
  }
});

test("the compiled server answers over HTTP on loopback without a token", async () => {
  const probe = createNetServer().listen(0, "127.0.0.1");
  await once(probe, "listening");
  const { port } = probe.address() as AddressInfo;
  probe.close();

  const child = spawn(process.execPath, [DIST_INDEX], {
    env: { PATH: process.env.PATH, MCP_TRANSPORT: "http", MCP_CLUSTER_WORKERS: "1", HOST: "127.0.0.1", PORT: String(port) },
    stdio: ["ignore", "ignore", "pipe"],
  });
  try {
    await new Promise<void>((resolve, reject) => {
      let stderr = "";
      child.stderr.on("data", (chunk: Buffer) => {
        stderr += chunk;
        if (stderr.includes("serving over http")) resolve();
      });
      child.on("exit", (code) => reject(new Error(`exited ${code}: ${stderr}`)));
    });
    const health = await fetch(`http://127.0.0.1:${port}/healthz`);
    assert.equal(health.status, 200);
    assert.equal(((await health.json()) as { version: string }).version, version);
  } finally {
    child.kill("SIGTERM");
    await once(child, "exit");
  }
});
