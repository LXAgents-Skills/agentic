# Overview

`LXAgents-Skills/agentic` is a read-only Skill MCP server. Every markdown file in `content/` is
served as its own tool, named `lxagents_skills_agentic_{base}`, and none of them takes an
argument. A client reads `lxagents_skills_agentic_automation` first; it lists every other tool
and the condition that activates it.

The set is for **fully autonomous, agentic workflows**. It has five tools: a workspace
resolver, a decision maker, a self corrector, an auto merger and a state manager. Each one
carries a failsafe, and all of them keep their work state outside the repository. See
[The agentic pipeline](agentic-pipeline.md).

| Surface | How |
|---|---|
| Skill MCP over stdio | `npm start` |
| Skill MCP over HTTP | `npm run start:http` (loopback, no token) or a container (token required) |
| Command line | `node dist/cli.js tools`, `--help`, `--version` |

The tools are derived from `content/` at boot. Adding a file adds a tool; there is no tool code
to write. Nothing writes, and nothing reaches a network.
