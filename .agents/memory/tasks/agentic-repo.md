---
name: memory-tasks-agentic-repo
description: Bootstrapping LXAgents-Skills/agentic from the template - five SRP agentic tools whose work state lives in a workspace outside the repository.
---

# Task: bootstrap the agentic Skill MCP repository

**Status: in progress.**

`LXAgents-Skills/agentic` serves instructions for fully autonomous, agentic workflows. One
file in `content/` is one purpose and one MCP tool. It is built from
`LXAgents-Skills/template` with `npm run init`, not written from scratch.

## What changes

- The repository is refreshed from the current template master and renamed with
  `npm run init -- --org LXAgents-Skills --repo agentic`: server ID `lxagents-agentic`,
  tool prefix `lxagents_skills_agentic_`.
- `content/example/` is replaced by five tools in `content/agentic/`:

  | File | Tool ID | Purpose |
  |---|---|---|
  | `workspace.md` | `lxagents_skills_agentic_workspace` | Where agentic work state is written: outside the repository |
  | `decision-maker.md` | `lxagents_skills_agentic_decision_maker` | Resolve ambiguity, with a Decision Audit Trail |
  | `self-corrector.md` | `lxagents_skills_agentic_self_corrector` | Measurable Done Criteria and a 3-retry Circuit Breaker |
  | `auto-merger.md` | `lxagents_skills_agentic_auto_merger` | Rebase and merge the stack, overriding the shared PR and merge gates, with a Conflict Resolution Protocol |
  | `state-manager.md` | `lxagents_skills_agentic_state_manager` | Handover Artifacts between steps |

- `content/automation.md` lists all five; the existing hub test covers it.
- A test fails if any tool tells an agent to write into `.agents/memory/` or the repository.
- README and `wiki/` describe the pipeline, the SRP design and the failsafes.
- Release `0.1.0`, the version the owner's spec names.

## What does not change

- The server code, the ID rule, HTTP auth (token only off loopback), install by git clone.
- `LXAgents-Skills/template`, `shared-instruction` and `security`.

## The owner's corrections to the spec

- **Work state is never written into the repository.** The spec put the decision log and
  handover in `.agents/memory/`. They go to `{workspace}/agentic/{org}/{repo}/` instead, a
  folder that is not inside any git work tree. The workspace is resolved by
  `lxagents_skills_agentic_workspace`.
- **Tool IDs follow `{org}_{repo}_{base}`**, underscores only, at most 64 characters, so
  `lxagents_skills_agentic_decision_maker`, not `agentic_decision_maker`.

## The plan

| # | Task | Branch | PR |
|---|---|---|---|
| 1 | This record | `plan/agentic-repo` | |
| 2 | Refresh from the template and run `init` | `feat/init` | |
| 3 | The five tools, the hub, the content test | `feat/agentic-tools` | |
| 4 | README, wiki, agent map | `docs/agentic-pipeline` | |
| 5 | Version 0.1.0, changelog, close this record | `release/0.1.0` | |

## Decisions taken without the owner

- **A fifth tool, `workspace`.** The workspace location is one fact that two tools need. It
  gets one home, and the hub tells an agent to read it before either tool writes.
- **This record stays in `.agents/memory/tasks/`.** The shared plan convention requires the
  development record of the repository itself. The owner's correction is about what the
  served tools write at run time, which never goes in a repository.
- **The harness's `claude/…` branch name is set aside**, as the shared branching rule requires.

## Progress

### Task 1 — plan/agentic-repo

Landed: this record and its row in `memory-index.md`.
