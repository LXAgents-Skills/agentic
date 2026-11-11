---
name: self-corrector
description: Prove a finished task works with commands, not judgment - Measurable Done Criteria, then fix and re-verify with a Circuit Breaker of at most 3 retries before escalating to a human.
---

# Self Corrector

A task is not done because it looks done. It is done when a command says so. This tool runs
after a task is built and before it is called done, committed as done, or handed to a merge.

## Measurable Done Criteria

Before you verify, write the criteria down as **commands, each with the result that means
pass**.

* **Find the repository's own checks first.** Look in `package.json` scripts, a `Makefile`,
  `pyproject.toml`, the CI workflow files, and the contributor docs. Use the commands CI runs.
* **Cover the change.** Include, for example:
  * the test suite, or the part of it that exercises the change;
  * the linter and the formatter check;
  * the type checker;
  * the build.
* **At least one criterion must exercise the new behaviour.** If no existing test does, write
  one. A suite that passes without touching your change proves nothing about it.
* **Pass means exit code 0, and output you read.** "No tests found", a skipped suite, or a
  command that did not run all count as **fail**.
* **"I reviewed it and it looks right" is not a criterion.** Neither is reasoning, a dry run
  you did not execute, or another agent's claim.
* If the repository has no automated check at all, and you cannot add one that exercises the
  change, the task cannot be declared done. Escalate.

Run the criteria once **before** your change if you can, so that a failure you did not cause
is known in advance. You report a failure that was already there. You do not fix it inside
this task.

## The loop

1. **Verify.** Run every criterion.
2. **If all pass, the task is done.** Record the commands and their results as the evidence.
3. **If any fail, that is a retry:**
   1. Read the output and find the cause. Do not guess.
   2. Fix the cause in the code under change.
   3. Run **every** criterion again, not just the one that failed.

## Circuit Breaker

**At most 3 retries per task.** The first verification is not a retry. Each fix followed by a
re-run is one retry. The count belongs to the task, so changing your approach does not reset
it.

The breaker trips, and you stop, when any of these happens:

* the third retry still fails;
* the only way to pass would change the criteria;
* the failure is outside the code, for example a missing credential, a network outage, an
  unavailable service, or a permission you do not have.

**When it trips:**

1. Stop changing code. Leave the branch in its last state and do not merge it.
2. Write an escalation for the human. It holds the criteria, each attempt (what you changed
   and the result), the last failing output, your best hypothesis, and exactly what you need
   from them.
3. Record the escalation in the run's handover. The handover lives in the agentic workspace
   at `{workspace}/agentic/{org}/{repo}/handover.md`, a folder outside every git work tree,
   and never in the repository.
4. Tell the user, and wait for them. A tripped breaker is the one point in an autonomous run
   that is never decided alone.

## Never, to get a pass

These are not fixes. Each one is a reason to trip the breaker:

* skipping, deleting, disabling or quarantining a test;
* loosening an assertion, or updating a snapshot or expected output to match wrong output;
* lowering a lint rule, a type strictness, or a coverage threshold;
* bypassing hooks (`--no-verify`) or CI;
* catching and silencing the error that the test exposed.
