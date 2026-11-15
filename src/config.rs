//! Every environment variable the server reads, read in one place.
//!
//! The one exception is `MCP_AUTH_TOKEN`, which `http::auth` reads at call time on purpose: a
//! secret captured once at load is whatever was set then.

use std::collections::BTreeMap;

/// The environment, as name to value. `main` builds it once from the process.
pub type Env = BTreeMap<String, String>;

/// The server's configuration, as `load_config` returns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `MCP_TRANSPORT`, lowercased; `stdio` when unset.
    pub transport: String,
    /// `PORT`; 3000 when unset, `None` when set to something that is not a port.
    pub port: Option<u16>,
    /// `HOST`; `0.0.0.0` when unset or empty. The bind address is named rather than implied.
    pub host: String,
    /// Whether `host` is loopback. A local HTTP server needs no token.
    pub local: bool,
    /// `MCP_CLUSTER_WORKERS`: HTTP worker threads, one per CPU unless set.
    pub workers: usize,
    /// `MCP_ALLOWED_HOSTS`. Empty means no `Host` check is applied rather than one guessed at.
    pub allowed_hosts: Vec<String>,
}

impl Config {
    /// Whether the configured transport is HTTP.
    pub fn is_http(&self) -> bool {
        self.transport == "http" || self.transport == "streamable-http"
    }
}

/// The leading integer of `raw`, as JavaScript's `parseInt(raw, 10)` reads it.
///
/// The server read these variables with `parseInt` before it was Rust, so `"8080 "` and
/// `"3abc"` keep meaning what they meant.
fn leading_int(raw: &str) -> Option<i64> {
    let text = raw.trim_start();
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    let end = digits
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse::<i64>().ok().map(|value| sign * value)
}

/// A comma-separated list, trimmed, with empty entries dropped. Unset is empty.
fn split_list(raw: Option<&String>) -> Vec<String> {
    raw.map(|text| {
        text.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(String::from)
            .collect()
    })
    .unwrap_or_default()
}

/// How many HTTP worker threads to run: `MCP_CLUSTER_WORKERS` when it is a whole number of
/// zero or more, otherwise `cpus`. `0` and `1` both mean one thread.
fn worker_count(raw: Option<&String>, cpus: usize) -> usize {
    match raw.and_then(|text| leading_int(text)) {
        Some(value) if value >= 0 => usize::try_from(value).unwrap_or(cpus),
        _ => cpus,
    }
}

/// Whether `host` is a loopback address, which only this machine can reach.
///
/// `localhost`, any `127.x.x.x`, and `::1` with or without brackets. `0.0.0.0` and `::` mean
/// every interface, so they are not loopback; neither is an empty value.
pub fn is_loopback(host: &str) -> bool {
    let value = host.trim().to_lowercase();
    if value == "localhost" || value == "::1" || value == "[::1]" {
        return true;
    }
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 4
        && parts[0] == "127"
        && parts[1..]
            .iter()
            .all(|part| (1..=3).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit()))
}

/// The number of CPUs this process was given.
pub fn available_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}

/// The server's configuration, from `env`.
pub fn load_config(env: &Env) -> Config {
    let host = env
        .get("HOST")
        .filter(|value| !value.is_empty())
        .cloned()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let port = match env.get("PORT") {
        None => Some(3000),
        Some(raw) => leading_int(raw).and_then(|value| u16::try_from(value).ok()),
    };
    Config {
        transport: env
            .get("MCP_TRANSPORT")
            .cloned()
            .unwrap_or_else(|| "stdio".to_string())
            .to_lowercase(),
        port,
        local: is_loopback(&host),
        host,
        workers: worker_count(env.get("MCP_CLUSTER_WORKERS"), available_parallelism()),
        allowed_hosts: split_list(env.get("MCP_ALLOWED_HOSTS")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> Env {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn an_empty_environment_gives_the_documented_defaults() {
        assert_eq!(
            load_config(&Env::new()),
            Config {
                transport: "stdio".into(),
                port: Some(3000),
                host: "0.0.0.0".into(),
                local: false,
                workers: available_parallelism(),
                allowed_hosts: vec![],
            }
        );
    }

    #[test]
    fn every_variable_is_read_and_the_transport_is_case_insensitive() {
        let config = load_config(&env(&[
            ("MCP_TRANSPORT", "HTTP"),
            ("PORT", "8080"),
            ("HOST", "127.0.0.1"),
            ("MCP_CLUSTER_WORKERS", "1"),
            ("MCP_ALLOWED_HOSTS", " a.example , ,b.example,"),
        ]));
        assert_eq!(config.transport, "http");
        assert!(config.is_http());
        assert_eq!(config.port, Some(8080));
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.workers, 1);
        assert_eq!(config.allowed_hosts, vec!["a.example", "b.example"]);
    }

    #[test]
    fn an_unusable_worker_count_falls_back_to_one_per_cpu() {
        for value in ["", "-1", "many"] {
            assert_eq!(
                load_config(&env(&[("MCP_CLUSTER_WORKERS", value)])).workers,
                available_parallelism(),
                "{value:?}"
            );
        }
    }

    #[test]
    fn a_port_that_is_not_a_port_is_none() {
        assert_eq!(load_config(&env(&[("PORT", "nope")])).port, None);
        assert_eq!(load_config(&env(&[("PORT", "70000")])).port, None);
        assert_eq!(load_config(&env(&[("PORT", "8080 ")])).port, Some(8080));
    }

    #[test]
    fn only_a_loopback_bind_counts_as_local() {
        for host in ["127.0.0.1", "127.1.2.3", "localhost", "LOCALHOST", "::1", "[::1]"] {
            assert!(is_loopback(host), "{host}");
            assert!(load_config(&env(&[("HOST", host)])).local, "{host}");
        }
        for host in [
            "0.0.0.0",
            "::",
            "",
            "10.0.0.5",
            "192.168.1.2",
            "example.com",
            "127.0.0.1.example.com",
        ] {
            assert!(!is_loopback(host), "{host}");
        }
        assert!(
            !load_config(&Env::new()).local,
            "the default bind, every interface, is a server"
        );
    }
}
