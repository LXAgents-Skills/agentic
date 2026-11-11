---
name: Read this tool every session
description: Read this tool every session, first — it lists every tool and the one condition that activates each, so you load only what the request needs.
---

# Automation

Read this once, at the start of every session. It is the only tool you read unprompted.

* Call no other tool yet. Match each step of the request against the conditions below, and
  call a tool only when its condition is true, and only that tool.
* Never call a tool "just in case". Each tool is complete on its own, so one call gives you
  everything that tool has.
* Several conditions can be true at once. Call them in the order the work reaches them, not
  all at the start.
* If a tool named here is not in your tool list, say so and work from what you have. Never
  rebuild a missing tool from memory.
* No tool takes an argument.
* Work state from these tools is never written into a repository.

## Before writing any work state

- `lxagents_skills_agentic_workspace` — a run is about to write its first decision, handover or escalation; read it once per run, before any tool below writes.

## During a run

- `lxagents_skills_agentic_state_manager` — a run starts, a step ends, work passes to another agent or tool, or the run stops or escalates.
- `lxagents_skills_agentic_decision_maker` — you face a question, an ambiguity or a blocker you would otherwise ask the user about.
- `lxagents_skills_agentic_self_corrector` — a task is built and you are about to call it done, commit it as done, or hand it to a merge.

## Landing the work

- `lxagents_skills_agentic_auto_merger` — the user or the project's standing instructions direct the run to merge its own branches without review, and every branch has passed its done criteria.
