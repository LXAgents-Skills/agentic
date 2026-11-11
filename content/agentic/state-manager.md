---
name: state-manager
description: Write and read Handover Artifacts so the next agent or tool continues a run where the last one stopped - current state, completed work with evidence, and next steps - kept outside the repository.
---

# State Manager

Each tool in an autonomous pipeline does one thing, so no tool remembers what the one before
it did. The **Handover Artifact** carries that memory. Each step writes it, and the next step
reads it first.

## Where it lives

The handover is `handover.md` in the agentic workspace, at
`{workspace}/agentic/{org}/{repo}/handover.md`. The workspace is a folder outside every git
work tree. It is resolved, in order, from `AGENTIC_WORKSPACE`, from a shared folder the user
or host declares, or from the folder that holds the repository clone. **Never write the
handover into the repository, and never into `.agents/memory/`.** Work state is not the
product.

The file always holds the **current** state, overwritten in place. The history of why things
happened is in the append-only decision log beside it, not here.

## When to write it

* at the end of every step, before the next step or tool starts;
* before handing work to another agent;
* when the run escalates to a human or stops for any reason;
* before your context is likely to be compacted or the session to end.

## When to read it

At the start of every step and every session in a run, **before any other work**. Then check
it against the repository: `git rev-parse HEAD` on the recorded branch must match the recorded
sha. If it does not, the repository is the truth. Note the drift in the next handover and
continue from what is actually there.

## The artifact

```markdown
# Handover — {org}/{repo}

- **Updated:** {UTC timestamp, ISO 8601}
- **Run goal:** {one sentence, from the user's request}
- **Status:** in progress | blocked | escalated | done
- **Branch / HEAD:** {branch} @ {short sha}

## Completed
- {task} — {evidence: commit sha, pull request, check command and result}

## In flight
- {what is half done, and exactly where it stopped}

## Next steps
1. {the next action, written so it can be executed without rereading this conversation}
2. …

## Blockers and escalations
- {what is blocked, what is needed, and from whom}

## Retry counters
- {task}: {retries used} of 3

## Do not redo
- {work that is finished and verified, so the next agent does not repeat it}
```

## Rules

* **Facts, not narrative.** Every completed item carries evidence that someone else can
  check, such as a sha, a pull request, or a command and its result.
* **Next steps are executable.** "Continue the work" is not a next step.
* Keep the retry counters accurate, because the next step enforces the 3-retry limit from
  them.
* Never write a secret, a token, or personal data.
* When the run ends, set **Status** to `done` or `escalated`, and leave the file in place for
  the person who reviews the run.
