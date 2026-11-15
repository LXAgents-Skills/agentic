//! The command line, parsed into a `Command`. Pure: no effects, no environment.

use crate::{config::Env, identity::SERVER_ID};

/// What the arguments ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run the server, with these environment overrides from `serve` flags.
    Serve(Env),
    /// Print every tool ID, its length, and its description.
    Tools,
    /// Print the version.
    Version,
    /// Print the help; the exit code is 0.
    Help,
    /// An unknown command; print a hint and exit 2.
    Unknown(String),
}

/// The help text.
pub fn help() -> String {
    format!(
        "{SERVER_ID} - a Skill MCP instruction set, served read-only

Usage
  {SERVER_ID} [command]

Commands
  (none)           Run the Skill MCP server, as the environment says
  serve            Run the Skill MCP server
                   [--stdio | --http] [--port N] [--host ADDR]
  tools            List every tool ID, its length, and its description

Options
  -h, --help       Show this help
  -v, --version    Show the version

Environment
  MCP_TRANSPORT        stdio (default) or http
  MCP_AUTH_TOKEN       bearer token, 32+ characters; required for http unless HOST is
                       loopback (127.0.0.1, localhost, ::1); unused by stdio
  PORT                 HTTP port, default 3000
  HOST                 HTTP interface, default 0.0.0.0
  MCP_ALLOWED_HOSTS    comma-separated Host allow-list
  MCP_CLUSTER_WORKERS  HTTP worker threads, default one per CPU; 1 runs one thread"
    )
}

/// The value after `flag` in `args`, unless it is missing or empty.
fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a String> {
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1).filter(|value| !value.is_empty())
}

/// Environment overrides from `serve` flags. `--stdio` wins over `--http` when both are given.
fn serve_overrides(args: &[String]) -> Env {
    let mut overrides = Env::new();
    if args.iter().any(|arg| arg == "--http") {
        overrides.insert("MCP_TRANSPORT".into(), "http".into());
    }
    if args.iter().any(|arg| arg == "--stdio") {
        overrides.insert("MCP_TRANSPORT".into(), "stdio".into());
    }
    if let Some(port) = flag_value(args, "--port") {
        overrides.insert("PORT".into(), port.clone());
    }
    if let Some(host) = flag_value(args, "--host") {
        overrides.insert("HOST".into(), host.clone());
    }
    overrides
}

/// Parse the arguments after the program name.
pub fn parse(args: &[String]) -> Command {
    let Some((command, rest)) = args.split_first() else {
        return Command::Serve(Env::new());
    };
    match command.as_str() {
        "-v" | "--version" => Command::Version,
        "-h" | "--help" | "help" => Command::Help,
        "serve" => Command::Serve(serve_overrides(rest)),
        "tools" => Command::Tools,
        other => Command::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_string()).collect()
    }

    #[test]
    fn no_arguments_serves_as_the_environment_says() {
        assert_eq!(parse(&[]), Command::Serve(Env::new()));
    }

    #[test]
    fn serve_flags_become_environment_overrides() {
        let Command::Serve(env) = parse(&args(&[
            "serve",
            "--http",
            "--port",
            "8080",
            "--host",
            "127.0.0.1",
        ])) else {
            panic!("not serve");
        };
        assert_eq!(env["MCP_TRANSPORT"], "http");
        assert_eq!(env["PORT"], "8080");
        assert_eq!(env["HOST"], "127.0.0.1");

        let Command::Serve(env) = parse(&args(&["serve", "--http", "--stdio", "--port"])) else {
            panic!("not serve");
        };
        assert_eq!(env["MCP_TRANSPORT"], "stdio");
        assert!(!env.contains_key("PORT"));
    }

    #[test]
    fn the_other_commands_parse() {
        assert_eq!(parse(&args(&["tools"])), Command::Tools);
        assert_eq!(parse(&args(&["-v"])), Command::Version);
        assert_eq!(parse(&args(&["--version"])), Command::Version);
        assert_eq!(parse(&args(&["help"])), Command::Help);
        assert_eq!(parse(&args(&["-h"])), Command::Help);
        assert_eq!(parse(&args(&["nope"])), Command::Unknown("nope".into()));
    }
}
