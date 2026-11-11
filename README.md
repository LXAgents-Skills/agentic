# LXAgents-Skills/agentic

A read-only Skill MCP server for **fully autonomous, agentic workflows**. An agent that
connects to it gets the instructions to decide, verify, merge and hand over work without
waiting for a human, and the failsafes that keep that safe.

- **Server ID:** `lxagents-agentic`
- **Tool prefix:** `lxagents_skills_agentic_`
- **Start here:** `lxagents_skills_agentic_automation` lists every tool and when to call it.

## The pipeline

```text
workspace ─ find where work state goes (outside every repository)
    │
state_manager ─ read the last handover
    │
  work ── decision_maker (whenever the way forward is unclear)
    │
self_corrector ─ prove it with commands; at most 3 retries
    │
state_manager ─ write the handover
    │
auto_merger ─ rebase and merge the stack in order
```

## One file, one purpose, one tool

The design follows the Single Responsibility Principle. Every markdown file in `content/` has
one purpose and is served as one tool, named `{org}_{repo}_{base}` from its filename. No tool
takes an argument, and no tool points to another. A tool that needs a fact from elsewhere
states it itself. The hub, `content/automation.md`, is the only router. It gives each tool the
one condition that calls it, and a test fails if a tool has no row there.

| Tool | Serves | Purpose |
|---|---|---|
| `lxagents_skills_agentic_automation` | `content/automation.md` | The hub |
| `lxagents_skills_agentic_workspace` | `content/agentic/workspace.md` | Where work state is written, outside the repository |
| `lxagents_skills_agentic_decision_maker` | `content/agentic/decision-maker.md` | Decide, log, then act |
| `lxagents_skills_agentic_self_corrector` | `content/agentic/self-corrector.md` | Verify with commands, retry at most 3 times |
| `lxagents_skills_agentic_auto_merger` | `content/agentic/auto-merger.md` | Rebase and merge without review |
| `lxagents_skills_agentic_state_manager` | `content/agentic/state-manager.md` | Hand over between steps |

## The failsafes

| Failsafe | Tool | What it guarantees |
|---|---|---|
| **Decision Audit Trail** | decision maker | Every autonomous choice is logged (question, options, choice, reasoning, how to undo) **before** it is executed. No entry, no action. Irreversible options are escalated, never chosen. |
| **Measurable Done Criteria** | self corrector | "Done" means a test, lint, type or build command exited 0 and exercised the change. Judgment is not a criterion. |
| **Circuit Breaker** | self corrector | At most 3 retries per task. Then the agent stops and escalates. Skipping a test or loosening a check to get a pass trips it. |
| **Conflict Resolution Protocol** | auto merger | Only simple, non-overlapping conflicts are resolved alone. Structural ones, or any doubt, abort the rebase and escalate. |
| **Hard limits** | auto merger | The tool overrides the shared pull request and merge gates, so human review is bypassed. It still never merges on red checks, bypasses branch protection, or force-pushes the default branch. |
| **Handover Artifacts** | state manager | Each step writes the current state, completed work with evidence, and executable next steps. The next step reads them first and checks them against `HEAD`. |

### Work state never goes in the repository

The decision log and the handover describe a run in flight. They are written to
`{workspace}/agentic/{org}/{repo}/`, a folder outside every git work tree, and never to
`.agents/memory/` or anywhere else in a repository. The workspace is the first of these that
is outside git:

1. `AGENTIC_WORKSPACE`;
2. a shared folder the user, the project or the host declares;
3. the folder that holds the repository clone.

If none qualifies, the agent stops and asks.

## Quick start

Install by cloning; the package is not published.

PowerShell:

```powershell
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic; cd lxagents-agentic; npm install; npm test
```

Bash or Zsh:

```bash
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic && cd lxagents-agentic && npm install && npm test
```

Then `npm run tools` lists every tool, and `npm start` serves them over stdio.

stdio needs no token. `npm run start:http` serves HTTP on `127.0.0.1:3000`, also without one.
HTTP on any other address needs `MCP_AUTH_TOKEN`.

## Documentation

- [The agentic pipeline](wiki/information/agentic-pipeline.md)
- [Install by git clone](wiki/guides/install-by-git-clone.md)
- [Add a tool](wiki/guides/add-a-tool.md)
- [Environment](wiki/environments/env.md) · [Docker](wiki/environments/docker.md)
- [Architecture](wiki/information/architecture.md)
- Agents: [`AGENTS.md`](AGENTS.md)

## Licence

MIT — see [`LICENSE`](LICENSE).
