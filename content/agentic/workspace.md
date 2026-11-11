---
name: workspace
description: Where an autonomous run writes its work state (decision log, handover, escalations) - a workspace folder outside every git repository, never .agents/memory/ - and how to find it.
---

# Agentic Workspace

An autonomous run produces **work state**: the decision log, the handover between steps, and
escalation notes. Work state describes a run in flight. It is not the product, so it is
**never written into a repository**.

## The rule

* Work state goes in the **agentic workspace**, a folder that is not inside any git work tree.
* Never write work state into a repository. That means not in `.agents/memory/`, not in any
  other tracked path, and not in an untracked or git-ignored file inside a work tree either.
  An ignored file is still one `git add -f` or one changed `.gitignore` away from history.
* Never write it to a temporary folder such as `/tmp` or `%TEMP%`. The next agent has to find
  it, and a temporary folder can be gone by then.

## Finding the workspace

Try each candidate in order and take the **first one that passes the check** below.

1. **`AGENTIC_WORKSPACE`**, if it is set to an absolute path. Read it as
   `$AGENTIC_WORKSPACE` in Bash or Zsh, or `$env:AGENTIC_WORKSPACE` in PowerShell.
2. **A folder the user, the project instructions, or the host declares for shared work
   products.** One example is a project's shared files folder that outlives a session, such
   as `/mnt/project-files/`. Use it only if something you were given names it. Do not search
   the disk for one.
3. **The folder that holds the repository clone**, which is the parent of
   `git rev-parse --show-toplevel`.

**The check.** The candidate exists, or can be created, and you can write to it. It is also
not inside a git work tree. Run `git -C <candidate> rev-parse --is-inside-work-tree`. The
candidate passes only if that command fails or prints `false`. If it prints `true`, the
candidate is inside a repository, so discard it and try the next one.

**If no candidate passes, stop.** Ask the user for a workspace and do not write any work
state until you have one. Do not fall back to the repository.

## Layout

```text
{workspace}/agentic/{org}/{repo}/
  decision-log.md   append-only; one entry per autonomous decision or conflict resolution
  handover.md       the current handover artifact; overwritten in place at each step
```

* `{org}/{repo}` is the repository's forge slug as written there, for example
  `LXAgents-Skills/agentic`, taken from `git remote get-url origin`. A repository with no
  remote uses `local/{name of its top-level folder}`.
* A workspace can be shared by many repositories and many agents. Write only inside your own
  `{org}/{repo}/` folder. Re-read a file just before you change it, and append rather than
  rewrite where the layout says so.
* Create the folders on first use.

## What never goes in

Secrets, tokens, credentials, connection strings, private keys, or personal data about
anyone. Name the variable or the secret store that holds a value, never the value itself.

## Say where it is

In your first message of a run, and in the final report, state the resolved workspace path,
or state that none could be found. The user has to be able to find the log without asking.
