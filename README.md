# LXAgents-Skills/agentic

A read-only Skill MCP server: every markdown file in `content/` is served as its own tool.

- **Server ID:** `lxagents-agentic`
- **Tool prefix:** `lxagents_skills_agentic_`
- **Start here:** `lxagents_skills_agentic_automation` lists every tool and when to call it.


## Quick start

```powershell
git clone https://github.com/LXAgents-Skills/agentic.git lxagents-agentic; cd lxagents-agentic; npm install; npm test
npm run tools
npm start
```

stdio needs no token. `npm run start:http` serves HTTP on `127.0.0.1:3000`, also without one.
HTTP on any other address needs `MCP_AUTH_TOKEN`.

## Tools

| Tool | Serves |
|---|---|
| `lxagents_skills_agentic_automation` | `content/automation.md` — the hub |
| `lxagents_skills_agentic_workspace` | `content/agentic/workspace.md` |
| `lxagents_skills_agentic_decision_maker` | `content/agentic/decision-maker.md` |
| `lxagents_skills_agentic_self_corrector` | `content/agentic/self-corrector.md` |
| `lxagents_skills_agentic_auto_merger` | `content/agentic/auto-merger.md` |
| `lxagents_skills_agentic_state_manager` | `content/agentic/state-manager.md` |

Each ID is `{org}_{repo}_{base}`, at most 64 characters. No tool takes an argument.

## Documentation

- [Install by git clone](wiki/guides/install-by-git-clone.md)
- [Add a tool](wiki/guides/add-a-tool.md)
- [Environment](wiki/environments/env.md) · [Docker](wiki/environments/docker.md)
- [Architecture](wiki/information/architecture.md)
- Agents: [`AGENTS.md`](AGENTS.md)

## Licence

MIT — see [`LICENSE`](LICENSE).
