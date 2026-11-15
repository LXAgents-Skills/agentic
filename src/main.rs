//! Entry point: parse the arguments, read the environment once, then run what they name.
//!
//! Only this file writes to stdout, and only for CLI commands. On stdio, stdout is the
//! JSON-RPC channel, so every server-side line goes to stderr.

use std::process::ExitCode;

use lxagents_agentic::{
    cli::{self, Command},
    config::{Config, Env, load_config},
    http,
    identity::SERVER_ID,
    instructions,
    server::AgenticServer,
    stdio, version,
};

/// The tool list as the `tools` command prints it: ID, its length, description.
fn print_tools(server: &AgenticServer) {
    let tools = server.tool_list();
    let width = tools.iter().map(|tool| tool.name.len()).max().unwrap_or(0);
    for tool in tools {
        println!(
            "{:<width$}  {:>2}  {}",
            tool.name,
            tool.name.len(),
            tool.description.as_deref().unwrap_or_default()
        );
    }
}

/// A runtime sized for the transport: one thread for stdio and for `MCP_CLUSTER_WORKERS` of 0
/// or 1, otherwise that many worker threads.
fn runtime(config: &Config) -> std::io::Result<tokio::runtime::Runtime> {
    let workers = if config.is_http() { config.workers } else { 1 };
    if workers <= 1 {
        tokio::runtime::Builder::new_current_thread().enable_all().build()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(workers)
            .enable_all()
            .build()
    }
}

/// Run the transport `config` names.
fn serve(server: AgenticServer, config: &Config) -> Result<(), String> {
    let runtime = runtime(config).map_err(|error| format!("{SERVER_ID} cannot start a runtime: {error}"))?;
    if config.is_http() {
        runtime.block_on(http::serve(server, config))
    } else {
        runtime.block_on(stdio::serve(server))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = cli::parse(&args);

    match &command {
        Command::Version => {
            println!("{}", version::VERSION);
            return ExitCode::SUCCESS;
        }
        Command::Help => {
            println!("{}", cli::help());
            return ExitCode::SUCCESS;
        }
        Command::Unknown(name) => {
            eprintln!("Unknown command \"{name}\". Try --help.");
            return ExitCode::from(2);
        }
        Command::Serve(_) | Command::Tools => {}
    }

    // The set is checked once, here, so a malformed one stops the process before it answers.
    let tools = match instructions::surface::instructions() {
        Ok(tools) => tools,
        Err(problem) => {
            eprintln!("{SERVER_ID} cannot start: {problem}");
            return ExitCode::FAILURE;
        }
    };
    let server = AgenticServer::new(tools);

    let Command::Serve(overrides) = command else {
        print_tools(&server);
        return ExitCode::SUCCESS;
    };
    let mut env: Env = std::env::vars().collect();
    env.extend(overrides);
    match serve(server, &load_config(&env)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}
