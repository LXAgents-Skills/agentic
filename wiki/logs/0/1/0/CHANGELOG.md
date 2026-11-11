# 0.1.0

**Released 2026-11-11.** The first release of `LXAgents-Skills/agentic`: five single-purpose
tools for fully autonomous, agentic workflows, each with its own failsafe, and all of them
keeping their work state outside the repository.

## Added

- `lxagents_skills_agentic_workspace` (`content/agentic/workspace.md`): work state (the
  decision log, the handover, escalations) goes to `{workspace}/agentic/{org}/{repo}/`, a
  folder outside every git work tree. The workspace is resolved, in order, from
  `AGENTIC_WORKSPACE`, a shared folder the user or host declares, or the folder that holds
  the clone. If none of them qualifies, the run stops and asks.
- `lxagents_skills_agentic_decision_maker` (`content/agentic/decision-maker.md`): resolves
  ambiguity by picking the recommended option. Its Decision Audit Trail entry is appended to
  `decision-log.md` before the decision is executed. Irreversible options are escalated.
- `lxagents_skills_agentic_self_corrector` (`content/agentic/self-corrector.md`): Measurable
  Done Criteria (commands that exit 0 and exercise the change) and a Circuit Breaker of at
  most 3 retries, after which the run escalates.
- `lxagents_skills_agentic_auto_merger` (`content/agentic/auto-merger.md`): rebases and
  merges the run's own stack in order. **It overrides the shared pull request and merge
  gates**, under hard limits and a Conflict Resolution Protocol that aborts on structural
  conflicts.
- `lxagents_skills_agentic_state_manager` (`content/agentic/state-manager.md`): Handover
  Artifacts in `handover.md`, read first and written at every step.
- `content/automation.md` routes all five. `test/agentic-content.test.js` checks the
  failsafes and that no tool writes to `.agents/memory/`.
- `wiki/information/agentic-pipeline.md`: how a run fits together, for people reviewing one.

## Changed

- Built from `LXAgents-Skills/template` with `npm run init`: server `lxagents-agentic`, tool
  prefix `lxagents_skills_agentic_`, package `@lxagents-skills/agentic` (private, installed by
  git clone).
- The init end-to-end test no longer depends on the template's example tool.

## Removed

- The template's example tool, `content/example/example-guide.md`.

**What a reader has to do:** register the server (stdio needs no token). Set
`AGENTIC_WORKSPACE` if the host gives the agent no shared folder outside the repository.
