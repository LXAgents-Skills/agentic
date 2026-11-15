//! The agentic set: its five tools, the failsafe each carries, and the owner's rule that work
//! state is never written into a repository.

use lxagents_agentic::{identity::tool_id, instructions::surface::instructions};
use regex::Regex;

/// Each tool, by base, with the failsafe its text must define.
const FAILSAFES: [(&str, &str); 5] = [
    ("workspace", r"## Finding the workspace"),
    ("decision_maker", r"## Decision Audit Trail"),
    ("self_corrector", r"## Circuit Breaker[\s\S]*At most 3 retries"),
    (
        "auto_merger",
        r"## Conflict Resolution Protocol[\s\S]*git rebase --abort",
    ),
    ("state_manager", r"Handover Artifact"),
];

/// Tools that write work state, and the workspace file each writes.
const WRITERS: [(&str, &str); 4] = [
    ("decision_maker", "decision-log.md"),
    ("self_corrector", "handover.md"),
    ("auto_merger", "decision-log.md"),
    ("state_manager", "handover.md"),
];

/// The text a tool serves, by base name.
fn read(base: &str) -> &'static str {
    let name = tool_id(base);
    instructions()
        .unwrap()
        .iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("{name} is not served"))
        .text
}

#[test]
fn every_agentic_tool_is_served_under_org_repo_base() {
    for (base, _) in FAILSAFES {
        read(base);
    }
}

#[test]
fn every_agentic_tool_defines_its_failsafe() {
    for (base, failsafe) in FAILSAFES {
        assert!(Regex::new(failsafe).unwrap().is_match(read(base)), "{base}");
    }
}

#[test]
fn work_state_is_written_to_the_workspace_outside_the_repository() {
    for (base, file) in WRITERS {
        assert!(
            read(base).contains(&format!("{{workspace}}/agentic/{{org}}/{{repo}}/{file}")),
            "{base} must write {file} in the workspace"
        );
    }
}

#[test]
fn no_tool_names_agents_memory_except_to_forbid_it() {
    let forbids = Regex::new(r"(?i)\b(never|not)\b").unwrap();
    for tool in instructions().unwrap() {
        let lines: Vec<&str> = tool.text.split('\n').collect();
        for (at, line) in lines.iter().enumerate() {
            if !line.contains(".agents/memory") {
                continue;
            }
            // A wrapped sentence can put its "never" on the line before the path.
            let previous = if at == 0 { "" } else { lines[at - 1] };
            let sentence = format!("{previous} {line}");
            assert!(
                forbids.is_match(&sentence),
                "{} names .agents/memory/ without forbidding it: {}",
                tool.name,
                line.trim()
            );
        }
    }
}

#[test]
fn the_auto_merger_states_that_it_overrides_the_shared_gates() {
    let pattern = Regex::new(r"both gates are\s+overridden, and human review is bypassed").unwrap();
    assert!(pattern.is_match(read("auto_merger")));
}
