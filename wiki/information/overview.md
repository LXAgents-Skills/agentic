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
| Skill MCP over stdio | `target/release/lxagents-agentic`, no arguments |
| Skill MCP over HTTP | `lxagents-agentic serve --http --host 127.0.0.1` (loopback, no token) or a container (token required) |
| Command line | `lxagents-agentic tools`, `--help`, `--version` |

The server is Rust, built on the official MCP Rust SDK (`rmcp`) with `axum` for HTTP. The tools
are derived from `content/`, which is compiled into the binary and checked at boot. Adding a
file and rebuilding adds a tool; there is no tool code to write. Nothing writes, and nothing reaches a network.
