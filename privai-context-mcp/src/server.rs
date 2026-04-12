use crate::{config::Config, errors::Result, tools::TOOL_NAMES};
use async_trait::async_trait;
use rust_mcp_sdk::{
    *,
    error::SdkResult,
    mcp_server::{server_runtime, ServerHandler, McpServerOptions},
    schema::*,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServerRuntime {
    config: Config,
}

#[macros::mcp_tool(name = "privai_v0_get_reading_order", description = "Returns the canonical reading order for V0 specs")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct GetReadingOrderTool {}

#[macros::mcp_tool(name = "privai_v0_get_current_status", description = "Returns the status of a specific task or component")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct GetCurrentStatusTool {
    topic: String,
}

#[macros::mcp_tool(name = "privai_v0_lookup_direction", description = "Looks up the canonical V0 direction for a given topic")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct LookupDirectionTool {
    query: String,
    topic: Option<String>,
}

#[macros::mcp_tool(name = "privai_v0_lookup_control", description = "Looks up control plane / orchestration rules")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct LookupControlTool {
    query: String,
}

#[macros::mcp_tool(name = "privai_v0_get_guardrails", description = "Retrieves the system guardrails")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct GetGuardrailsTool {}

#[macros::mcp_tool(name = "privai_v0_route_question", description = "Routes a question to the right context layer")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct RouteQuestionTool {
    question: String,
}

#[macros::mcp_tool(name = "privai_v0_prepare_task_context", description = "Prepares full context for a given task")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct PrepareTaskContextTool {
    task_id: String,
}

#[macros::mcp_tool(name = "privai_v0_build_correction_pill", description = "Builds a correction pill if legacy framing is detected")]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, macros::JsonSchema)]
pub struct BuildCorrectionPillTool {
    legacy_concept: String,
}

impl ServerRuntime {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn serve_stdio(&self) -> Result<()> {
        self.validate_contract()?;
        
        let server_info = InitializeResult {
            server_info: Implementation {
                name: "privai-context-mcp".into(),
                version: "0.1.0".into(),
                title: Some("privAI Context Server".into()),
                description: Some("Read-only V0 context MCP server for privAI".into()),
                icons: vec![],
                website_url: None,
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                ..Default::default()
            },
            protocol_version: ProtocolVersion::V2025_11_25.into(),
            instructions: None,
            meta: None,
        };

        let transport = rust_mcp_sdk::StdioTransport::new(rust_mcp_sdk::TransportOptions::default())
            .map_err(|e| crate::errors::McpError::Unsupported(format!("{:?}", e)))?;
        
        let handler = PrivaiHandler { config: self.config.clone() };
        let handler = handler.to_mcp_server_handler();
        
        let options = McpServerOptions {
            server_details: server_info,
            transport,
            handler,
            task_store: None,
            client_task_store: None,
            message_observer: None,
        };

        let server = server_runtime::create_server(options);
        server.start().await.map_err(|e| crate::errors::McpError::Unsupported(e.to_string()))?;

        Ok(())
    }

    pub fn validate_contract(&self) -> Result<()> {
        if TOOL_NAMES.len() != 8 {
            return Err(crate::errors::McpError::InvalidConfig(format!(
                "expected exactly 8 tools, got {}",
                TOOL_NAMES.len()
            )));
        }
        Ok(())
    }
}

// ── MCP Server Handler ────────────────────────────────────────────

#[derive(Clone)]
struct PrivaiHandler {
    #[allow(dead_code)]
    config: Config,
}

#[async_trait]
impl ServerHandler for PrivaiHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: std::sync::Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        let tools = vec![
            GetReadingOrderTool::tool(),
            GetCurrentStatusTool::tool(),
            LookupDirectionTool::tool(),
            LookupControlTool::tool(),
            GetGuardrailsTool::tool(),
            RouteQuestionTool::tool(),
            PrepareTaskContextTool::tool(),
            BuildCorrectionPillTool::tool(),
        ];

        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: std::sync::Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Here you would connect your existing `store` (FileStore / VertexRAGStore)
        let response = match params.name.as_str() {
            "privai_v0_lookup_direction" => {
                format!("Lookup Direction called. Config root: {}", self.config.v0_root.display())
            },
            _ => format!("Tool '{}' executed successfully (placeholder response).", params.name)
        };

        Ok(CallToolResult::text_content(vec![response.into()]))
    }
}
