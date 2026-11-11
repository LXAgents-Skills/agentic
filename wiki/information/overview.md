# Overview

`LXAgents-Skills/agentic` is a read-only Skill MCP server. Every markdown file in `content/` is
served as its own tool, named `lxagents_skills_agentic_{base}`, and none of them takes an
argument. A client reads `lxagents_skills_agentic_automation` first; it lists every other tool
and the condition that activates it.


| Surface | How |
|---|---|
| Skill MCP over stdio | `npm start` |
| Skill MCP over HTTP | `npm run start:http` (loopback, no token) or a container (token required) |
| Command line | `node src/cli.js tools`, `--help`, `--version` |

The tools are derived from `content/` at boot. Adding a file adds a tool; there is no tool code
to write. Nothing writes, and nothing reaches a network.
