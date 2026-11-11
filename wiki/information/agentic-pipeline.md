# The Agentic Pipeline

How the five tools in `content/agentic/` fit together in one autonomous run, and what a person
reviewing the run should look at afterwards.

## A run, step by step

1. **Find the workspace** (`lxagents_skills_agentic_workspace`). The agent resolves a folder
   outside every git work tree and says where it is. The candidates, in order, are
   `AGENTIC_WORKSPACE`, a shared folder the user or host declares, and the folder that holds
   the clone. If none of them is outside git, the run stops before writing anything.
2. **Read the handover** (`lxagents_skills_agentic_state_manager`). If an earlier step left
   one, the agent continues from it after checking that the recorded `HEAD` matches the
   repository.
3. **Do the work.** Whenever the way forward is unclear, the decision maker
   (`lxagents_skills_agentic_decision_maker`) lists the options, picks the recommended one,
   logs it, and only then acts. It escalates instead of choosing when an option cannot be
   undone, for example production, deleting data or history, spending money, outside
   messages, or secrets.
4. **Prove it** (`lxagents_skills_agentic_self_corrector`). The agent writes the done
   criteria as commands and runs them. Each failure gets a fix and a full re-run, up to 3
   retries. After that, the breaker trips and a person is asked.
5. **Hand over** (`lxagents_skills_agentic_state_manager`). The agent writes the current
   state, the completed work with evidence, the next steps, and the retry counters.
6. **Merge** (`lxagents_skills_agentic_auto_merger`). Only when the user or the project has
   directed merging without review. The agent rebases each branch of the run's own stack onto
   the default branch, waits for green checks, merges in order, and verifies the default
   branch holds the stack.

## What it overrides, and what it does not

The shared instruction set asks for a person's yes before a pull request is opened and before
anything is merged. The auto merger overrides both of those gates. It does not override its
own hard limits:

- it never merges on failing or pending checks;
- it never bypasses branch protection;
- it never force-pushes the default branch or a branch it did not create;
- it never enables a forge's auto-merge in place of checking each merge.

## Where to look after a run

```text
{workspace}/agentic/{org}/{repo}/
  decision-log.md   every autonomous decision and conflict resolution, append-only
  handover.md       where the run stopped, and why
```

Neither file is ever in the repository. The repository holds only the work: the branches,
the pull requests, and the merged commits.

If the run escalated, `handover.md` has **Status: escalated**, and its *Blockers and
escalations* section says what is needed and from whom.
