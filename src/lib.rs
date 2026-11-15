//! `lxagents-agentic`: a Skill MCP server that serves every markdown file in `content/`
//! read-only as its own tool, over stdio or stateless Streamable HTTP.
//!
//! The binary in `main.rs` is the only effectful entry point; everything here can be built
//! and tested without binding a port or reading the process environment, except where a
//! function says so.

pub mod cli;
pub mod config;
pub mod http;
pub mod identity;
pub mod instructions;
pub mod server;
pub mod stdio;
pub mod version;
