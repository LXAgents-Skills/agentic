//! The built binary, as a client installs it: the CLI and a stdio handshake.

mod common;

use std::process::Command;

use common::{BIN, server};
use lxagents_agentic::version::VERSION;
use rmcp::{ServiceExt, transport::TokioChildProcess};

fn run(args: &[&str]) -> std::process::Output {
    Command::new(BIN).args(args).output().expect("run the binary")
}

#[test]
fn version_prints_the_cargo_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), VERSION);
}

#[test]
fn tools_prints_every_tool_in_order() {
    let output = run(&["tools"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let names: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let expected: Vec<String> = server()
        .tool_list()
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();
    assert_eq!(names, expected);
}

#[test]
fn help_exits_0_and_an_unknown_command_exits_2() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("MCP_AUTH_TOKEN"));
    let unknown = run(&["nope"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("Unknown command \"nope\""));
}

#[tokio::test]
async fn the_binary_answers_over_stdio_with_no_arguments() {
    let mut command = tokio::process::Command::new(BIN);
    command.env_remove("MCP_TRANSPORT");
    let (transport, _stderr) = TokioChildProcess::builder(command)
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn");
    let client = ().serve(transport).await.expect("handshake");
    let tools = client.list_all_tools().await.expect("tools/list");
    let names: Vec<String> = tools.into_iter().map(|tool| tool.name.to_string()).collect();
    let expected: Vec<String> = server()
        .tool_list()
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();
    assert_eq!(names, expected);
    client.cancel().await.expect("close");
}
