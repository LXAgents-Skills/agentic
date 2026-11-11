---
name: decision-maker
description: Resolve a question, ambiguity or blocker without waiting for a human - analyze the options, pick the recommended one, log a Decision Audit Trail entry, then execute it.
---

# Decision Maker

An autonomous run does not stop to ask when the way forward is unclear. It decides, writes
down why, and continues. This tool covers the decision. The audit entry is what makes the
decision safe to take alone, because a person reading the log later can check the reasoning
and undo the choice.

## When it applies

* The request can be read more than one way.
* A step has more than one reasonable way to do it, and the request does not say which one.
* You hit a blocker with more than one way past it.
* You would otherwise ask the user a question and wait.

## What it never decides alone

Some actions cannot be undone by reading a log. Stop and escalate to a human instead of
choosing, when an option would:

* change production, a live deployment, or shared infrastructure;
* delete data, a branch other than your own, or history (`git push --force` to a default
  branch, `git reset` on a shared branch);
* spend money, send a message outside the project, or publish a package or a release tag;
* create, rotate, read out or widen a secret or a permission.

Merging your own run's branches is not on this list. That has its own tool and its own
protocol.

## Procedure

1. **State the question** in one sentence.
2. **List the options.** Give two to four. For each one, say what happens if you choose it,
   whether it can be undone, and how.
3. **Analyze.** Weigh each option against these, in this order:
   1. the user's stated goal and instructions;
   2. the repository's own conventions, its existing code and its rules;
   3. reversibility, preferring the option that is easiest to undo;
   4. blast radius, preferring the option that touches the least;
   5. evidence, meaning what you checked rather than what you assume.
4. **Choose** the option that analysis recommends. If two options tie, take the more
   reversible one.
5. **Write the audit entry** (below). This step comes before you act, every time.
6. **Execute** the chosen option.
7. **Report** the decision in the run's final report, one line for each decision.

## Decision Audit Trail

Before you execute a decision, append an entry to `decision-log.md` in the agentic workspace.
That file lives at `{workspace}/agentic/{org}/{repo}/decision-log.md`, where the workspace is
a folder outside every git work tree. The workspace is resolved, in order, from
`AGENTIC_WORKSPACE`, from a shared folder the user or host declares, or from the folder that
holds the repository clone. **Never write the log into the repository, and never into
`.agents/memory/`.**

```markdown
## {UTC timestamp, ISO 8601} — {short title}

- **Repository / branch:** {org}/{repo} @ {branch} ({short HEAD sha})
- **Question:** {one sentence}
- **Options:**
  1. {option} — {consequence}; {reversible? how}
  2. {option} — {consequence}; {reversible? how}
- **Chosen:** {option number and name}
- **Reasoning:** {why, against the criteria above, citing what you checked}
- **To undo:** {the command or step that reverses it}
```

* **No entry, no action.** If the entry cannot be written, do not execute the decision.
  Escalate instead. The trail is the condition for acting alone, not a summary written
  afterwards.
* The log is append-only. To reverse an earlier decision, write a new entry that names the
  one it reverses. Never edit or delete a past entry.
* Never write a secret, a token, or personal data into an entry.
