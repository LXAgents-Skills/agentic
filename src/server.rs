//! The Skill MCP server.
//!
//! The surface is `instructions()` and nothing else. No tool takes an argument, a verb or a
//! credential, and nothing here writes or reaches a network: the code that would is absent.

use std::sync::Arc;

use rmcp::{
    ErrorData, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, JsonObject,
        ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    },
    service::{RequestContext, RoleServer},
};

use crate::{
    identity::{SERVER_ID, SERVER_SUMMARY, SERVER_TITLE, tool_id},
    instructions::surface::Instruction,
    version::VERSION,
};

/// The `instructions` a client receives in `initialize`.
pub fn server_instructions() -> String {
    [
        SERVER_SUMMARY.to_string(),
        format!(
            "Read {} once at the start of every session: it lists every tool and the one condition that activates each.",
            tool_id("automation")
        ),
        "Call no other tool until its condition is true. No tool takes an argument.".to_string(),
    ]
    .join(" ")
}

/// The input schema every tool has: an object with no properties.
fn no_arguments() -> Arc<JsonObject> {
    let mut schema = JsonObject::new();
    schema.insert("type".into(), "object".into());
    schema.insert("properties".into(), JsonObject::new().into());
    Arc::new(schema)
}

/// The server. Cheap to clone; every connection or HTTP request gets its own.
#[derive(Clone)]
pub struct AgenticServer {
    tools: &'static [Instruction],
}

impl AgenticServer {
    /// A server over `tools`, which `main` has already built and checked.
    pub fn new(tools: &'static [Instruction]) -> Self {
        Self { tools }
    }

    /// The registered tools, as MCP lists them.
    pub fn tool_list(&self) -> Vec<Tool> {
        let schema = no_arguments();
        self.tools
            .iter()
            .map(|tool| Tool::new(tool.name.clone(), tool.description.clone(), schema.clone()))
            .collect()
    }

    /// The result of calling `name`: its file as one text block, or a tool error.
    pub fn call(&self, name: &str) -> CallToolResult {
        match self.tools.iter().find(|tool| tool.name == name) {
            Some(tool) => CallToolResult::success(vec![ContentBlock::text(tool.text)]),
            None => CallToolResult::error(vec![ContentBlock::text(format!(
                "MCP error -32602: Tool {name} not found"
            ))]),
        }
    }
}

impl ServerHandler for AgenticServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
        )
        .with_server_info(Implementation::new(SERVER_ID, VERSION).with_title(SERVER_TITLE))
        .with_instructions(server_instructions())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tool_list()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        Ok(self.call(&request.name).into())
    }
}
