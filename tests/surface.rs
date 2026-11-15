//! The served surface. Every test is written against whatever `content/` holds.

mod common;

use std::collections::BTreeSet;

use common::{client, content_dir, markdown_files, server};
use lxagents_agentic::{
    identity::{SERVER_ID, SERVER_TITLE, tool_id, tool_prefix},
    instructions::{
        naming::{MAX_ID_LENGTH, OVERRIDE_BASES, tool_name_for},
        surface::instructions,
    },
    server::server_instructions,
    version::VERSION,
};
use rmcp::model::CallToolRequestParams;

fn hub() -> String {
    tool_id("automation")
}

#[tokio::test]
async fn the_server_names_itself_and_gives_its_instructions() {
    let client = client().await;
    let info = client.peer_info().expect("peer info");
    let implementation = info.server_info.as_ref().expect("server info");
    assert_eq!(implementation.name, SERVER_ID);
    assert_eq!(implementation.title.as_deref(), Some(SERVER_TITLE));
    assert_eq!(implementation.version, VERSION);
    assert_eq!(info.instructions.as_deref(), Some(server_instructions().as_str()));
    assert!(server_instructions().contains(&hub()));
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn the_cli_list_and_the_mcp_tool_list_agree() {
    let client = client().await;
    let listed = client.list_all_tools().await.expect("tools/list");
    let local = server().tool_list();
    assert_eq!(listed.len(), local.len());
    for (remote, local) in listed.iter().zip(&local) {
        assert_eq!(remote.name, local.name);
        assert_eq!(remote.description, local.description);
    }
    client.cancel().await.expect("close");
}

#[test]
fn every_markdown_file_is_one_tool_and_every_tool_is_one_file() {
    let served: Vec<&str> = instructions().unwrap().iter().map(|tool| tool.path).collect();
    assert_eq!(served, markdown_files());
}

#[test]
fn every_id_is_org_repo_base_underscores_only_at_most_64_characters() {
    for tool in instructions().unwrap() {
        let name = &tool.name;
        assert!(name.starts_with(&format!("{}_", tool_prefix())), "{name}");
        assert!(
            name.chars().next().is_some_and(|ch| ch.is_ascii_lowercase())
                && name
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
            "{name}"
        );
        assert!(name.len() <= MAX_ID_LENGTH, "{name} is {} characters", name.len());
        assert_eq!(*name, tool_name_for(tool.path, OVERRIDE_BASES), "{}", tool.path);
    }
}

#[tokio::test]
async fn no_tool_takes_an_argument() {
    let client = client().await;
    for tool in client.list_all_tools().await.expect("tools/list") {
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(|value| value.as_object());
        assert!(
            properties.is_some_and(|map| map.is_empty()),
            "{} takes an argument",
            tool.name
        );
        assert_eq!(
            tool.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object")
        );
    }
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn every_tool_returns_its_own_file_byte_for_byte() {
    let client = client().await;
    for tool in instructions().unwrap() {
        let result = client
            .call_tool(CallToolRequestParams::new(tool.name.clone()))
            .await
            .expect("tools/call");
        assert_ne!(result.is_error, Some(true), "{}", tool.name);
        assert_eq!(result.content.len(), 1, "{}", tool.name);
        let on_disk = std::fs::read_to_string(content_dir().join(tool.path)).expect("read");
        assert_eq!(
            result.content[0].as_text().map(|text| text.text.as_str()),
            Some(on_disk.as_str()),
            "{}",
            tool.name
        );
    }
    client.cancel().await.expect("close");
}

#[tokio::test]
async fn an_unknown_tool_is_a_tool_error() {
    let client = client().await;
    let result = client
        .call_tool(CallToolRequestParams::new("nope"))
        .await
        .expect("tools/call");
    assert_eq!(result.is_error, Some(true));
    assert_eq!(
        result.content[0].as_text().map(|text| text.text.as_str()),
        Some("MCP error -32602: Tool nope not found")
    );
    client.cancel().await.expect("close");
}

#[test]
fn the_automation_hub_is_served() {
    assert!(
        instructions().unwrap().iter().any(|tool| tool.name == hub()),
        "content/automation.md must derive {}",
        hub()
    );
}

#[test]
fn the_automation_hub_lists_every_other_tool_exactly_once_and_nothing_else() {
    let tools = instructions().unwrap();
    let hub_text = tools.iter().find(|tool| tool.name == hub()).expect("hub").text;
    let prefix = format!("- `{}_", tool_prefix());
    let rows: Vec<(&str, &str)> = hub_text
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .filter_map(|line| {
            let rest = &line[3..];
            let end = rest.find('`')?;
            Some((&rest[..end], &rest[end + 1..]))
        })
        .collect();

    let names: Vec<&str> = rows.iter().map(|(name, _)| *name).collect();
    let unique: BTreeSet<&str> = names.iter().copied().collect();
    assert_eq!(
        unique.len(),
        names.len(),
        "a tool is listed twice in automation.md"
    );

    let expected: BTreeSet<&str> = tools
        .iter()
        .map(|tool| tool.name.as_str())
        .filter(|name| *name != hub())
        .collect();
    assert_eq!(
        unique, expected,
        "content/automation.md must list every other tool and nothing else"
    );

    for (name, after) in rows {
        let condition = after.strip_prefix(" — ").unwrap_or("");
        assert!(
            condition.chars().next().is_some_and(|ch| !ch.is_whitespace()),
            "{name} needs a condition after \" — \""
        );
    }
}
