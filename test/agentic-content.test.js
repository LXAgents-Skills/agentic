import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { toolId } from "../src/identity.js";
import { INSTRUCTION_FILES } from "../src/instructions/from-content.js";
import { CONTENT_DIR } from "../src/paths.js";

/*
 * The agentic set: its five tools, the failsafe each one carries, and the owner's rule that
 * work state is never written into a repository.
 */

/** Each tool, by base, with the failsafe its text must define. */
const FAILSAFES = {
  workspace: /## Finding the workspace/,
  decision_maker: /## Decision Audit Trail/,
  self_corrector: /## Circuit Breaker[\s\S]*At most 3 retries/,
  auto_merger: /## Conflict Resolution Protocol[\s\S]*git rebase --abort/,
  state_manager: /Handover Artifact/,
};

/** Tools that write work state, and the workspace file each writes. */
const WRITERS = {
  decision_maker: "decision-log.md",
  self_corrector: "handover.md",
  auto_merger: "decision-log.md",
  state_manager: "handover.md",
};

const read = (base) => readFileSync(join(CONTENT_DIR, INSTRUCTION_FILES.get(toolId(base))), "utf8");

test("every agentic tool is served under {org}_{repo}_{base}", () => {
  for (const base of Object.keys(FAILSAFES)) {
    assert.ok(INSTRUCTION_FILES.has(toolId(base)), `${toolId(base)} is not served`);
  }
});

test("every agentic tool defines its failsafe", () => {
  for (const [base, failsafe] of Object.entries(FAILSAFES)) assert.match(read(base), failsafe, base);
});

test("work state is written to the workspace, outside the repository", () => {
  for (const [base, file] of Object.entries(WRITERS)) {
    assert.ok(read(base).includes(`{workspace}/agentic/{org}/{repo}/${file}`), `${base} must write ${file} in the workspace`);
  }
});

test("no tool names .agents/memory/ except to forbid it", () => {
  for (const [name, path] of INSTRUCTION_FILES) {
    const lines = readFileSync(join(CONTENT_DIR, path), "utf8").split("\n");
    lines.forEach((line, at) => {
      if (!line.includes(".agents/memory")) return;
      // A wrapped sentence can put its "never" on the line before the path.
      const sentence = `${lines[at - 1] ?? ""} ${line}`;
      assert.match(sentence, /\b(never|not)\b/i, `${name} names .agents/memory/ without forbidding it: ${line.trim()}`);
    });
  }
});

test("the auto merger states that it overrides the shared gates", () => {
  assert.match(read("auto_merger"), /both gates are\s+overridden, and human review is bypassed/);
});
