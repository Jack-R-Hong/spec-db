use std::path::PathBuf;

use rmcp::ErrorData as McpError;
use rmcp::RoleServer;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestMethod, CallToolRequestParams, CallToolResult, ErrorCode,
    GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
    ListResourcesResult, ListToolsResult, PaginatedRequestParams, ReadResourceRequestParams,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use crate::prompts;
use crate::resources::ResourceHandler;
use crate::tools::{
    AddCausalLinkInput, AddSpecInput, EdgeActionInput, FindDependenciesInput, GetSpecInput,
    QueryInput, SearchSpecsInput, SyncInput, ToolHandler, TraceImpactInput,
};

#[derive(Clone)]
pub struct SpecDbMcpServer {
    tools: ToolHandler,
    resources: ResourceHandler,
}

impl SpecDbMcpServer {
    pub fn new(
        repo_path: PathBuf,
        specs_root: String,
        tantivy_dir: PathBuf,
        fjall_dir: PathBuf,
        ai_default_trust: f64,
    ) -> Self {
        Self {
            tools: ToolHandler {
                repo_path,
                specs_root,
                tantivy_dir: tantivy_dir.clone(),
                fjall_dir: fjall_dir.clone(),
                ai_default_trust,
            },
            resources: ResourceHandler { tantivy_dir, fjall_dir },
        }
    }

    fn tool_definitions() -> Vec<Tool> {
        vec![
            Tool::new(
                "search_specs",
                "Search indexed specs",
                schema_object(json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1 },
                        "tags": { "type": "array", "items": { "type": "string" } }
                    }
                })),
            ),
            Tool::new(
                "get_spec",
                "Get a spec by id",
                schema_object(json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": { "id": { "type": "string" } }
                })),
            ),
            Tool::new(
                "trace_impact",
                "Trace impact from a node",
                schema_object(json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": { "type": "string" },
                        "depth": { "type": "integer", "minimum": 1 }
                    }
                })),
            ),
            Tool::new(
                "find_dependencies",
                "Find dependencies for a node",
                schema_object(json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": { "id": { "type": "string" } }
                })),
            ),
            Tool::new(
                "query",
                "Run natural language query routing",
                schema_object(json!({
                    "type": "object",
                    "required": ["natural_language"],
                    "properties": { "natural_language": { "type": "string" } }
                })),
            ),
            Tool::new(
                "add_spec",
                "Ingest a spec markdown document",
                schema_object(json!({
                    "type": "object",
                    "required": ["markdown"],
                    "properties": { "markdown": { "type": "string" } }
                })),
            ),
            Tool::new(
                "add_causal_link",
                "Add an AI-proposed causal edge between existing specs",
                schema_object(json!({
                    "type": "object",
                    "required": ["source", "target"],
                    "properties": {
                        "source": { "type": "string" },
                        "target": { "type": "string" },
                        "edge_type": {
                            "type": "string",
                            "enum": ["depends_on", "constrains", "implements"],
                            "default": "depends_on"
                        }
                    }
                })),
            ),
            Tool::new(
                "promote_edge",
                "Promote an AI-inferred edge to human-curated status",
                schema_object(json!({
                    "type": "object",
                    "required": ["source", "target"],
                    "properties": {
                        "source": { "type": "string" },
                        "target": { "type": "string" },
                        "edge_type": {
                            "type": "string",
                            "enum": ["depends_on", "constrains", "implements"],
                            "default": "depends_on"
                        }
                    }
                })),
            ),
            Tool::new(
                "reject_edge",
                "Reject and remove an AI-inferred edge from the graph",
                schema_object(json!({
                    "type": "object",
                    "required": ["source", "target"],
                    "properties": {
                        "source": { "type": "string" },
                        "target": { "type": "string" },
                        "edge_type": {
                            "type": "string",
                            "enum": ["depends_on", "constrains", "implements"],
                            "default": "depends_on"
                        }
                    }
                })),
            ),
            Tool::new(
                "sync",
                "Run incremental or full sync",
                schema_object(json!({
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["incremental", "full"]
                        }
                    }
                })),
            ),
        ]
    }
}

impl ServerHandler for SpecDbMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "lattice".to_owned(),
                version: "0.1.0".to_owned(),
                ..Implementation::default()
            },
            instructions: Some("Lattice MCP server over stdio".to_owned()),
            ..ServerInfo::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(Self::tool_definitions())))
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListPromptsResult::with_all_items(prompts::prompt_definitions())))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let tools = self.tools.clone();
        let handle = tokio::task::spawn_blocking(move || prompts::resolve_prompt(&tools, &request));

        let value = handle
            .await
            .map_err(|e| McpError::internal_error(format!("prompt task join failed: {e}"), None))?;

        match value {
            Ok(result) => Ok(result),
            Err(error) => {
                let payload = mcp_error_payload(error);
                Err(McpError::internal_error(payload.to_string(), None))
            }
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let args = request.arguments.unwrap_or_default();
        let tools = self.tools.clone();

        let handle = tokio::task::spawn_blocking(move || match tool_name.as_str() {
            "search_specs" => {
                parse_args::<SearchSpecsInput>(args).and_then(|input| tools.search_specs(input))
            }
            "get_spec" => parse_args::<GetSpecInput>(args).and_then(|input| tools.get_spec(input)),
            "trace_impact" => {
                parse_args::<TraceImpactInput>(args).and_then(|input| tools.trace_impact(input))
            }
            "find_dependencies" => parse_args::<FindDependenciesInput>(args)
                .and_then(|input| tools.find_dependencies(input)),
            "query" => parse_args::<QueryInput>(args).and_then(|input| tools.query(input)),
            "add_spec" => parse_args::<AddSpecInput>(args).and_then(|input| tools.add_spec(input)),
            "add_causal_link" => parse_args::<AddCausalLinkInput>(args)
                .and_then(|input| tools.add_causal_link(input)),
            "promote_edge" => {
                parse_args::<EdgeActionInput>(args).and_then(|input| tools.promote_edge(input))
            }
            "reject_edge" => {
                parse_args::<EdgeActionInput>(args).and_then(|input| tools.reject_edge(input))
            }
            "sync" => parse_args::<SyncInput>(args).and_then(|input| tools.sync(input)),
            _ => Err(spec_db_core::SpecDbError::ConfigError(format!("unknown tool: {tool_name}"))),
        });

        let value = handle
            .await
            .map_err(|e| McpError::internal_error(format!("tool task join failed: {e}"), None))?;

        match value {
            Ok(payload) => Ok(CallToolResult::structured(payload)),
            Err(spec_db_core::SpecDbError::ConfigError(message))
                if message.starts_with("unknown tool:") =>
            {
                Err(McpError::method_not_found::<CallToolRequestMethod>())
            }
            Err(error) => Ok(CallToolResult::structured_error(mcp_error_payload(error))),
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult::with_all_items(self.resources.list_resources())))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri;
        match self.resources.read_resource(&uri) {
            Ok(content) => Ok(ReadResourceResult { contents: vec![content] }),
            Err(spec_db_core::SpecDbError::ConfigError(message))
                if message.starts_with("resource not found:") =>
            {
                Err(McpError::new(ErrorCode::RESOURCE_NOT_FOUND, message, None))
            }
            Err(error) => {
                let payload = mcp_error_payload(error);
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(payload.to_string(), uri)],
                })
            }
        }
    }
}

fn parse_args<T: DeserializeOwned>(
    args: Map<String, Value>,
) -> Result<T, spec_db_core::SpecDbError> {
    serde_json::from_value(Value::Object(args))
        .map_err(|e| spec_db_core::SpecDbError::IngestError(format!("invalid tool arguments: {e}")))
}

fn schema_object(value: Value) -> std::sync::Arc<Map<String, Value>> {
    if let Value::Object(map) = value {
        std::sync::Arc::new(map)
    } else {
        std::sync::Arc::new(Map::new())
    }
}

fn mcp_error_payload(error: spec_db_core::SpecDbError) -> Value {
    if let Some(payload) = decode_custom_mcp_error(&error) {
        return payload;
    }

    let (error_type, message) = match error {
        spec_db_core::SpecDbError::SearchError(msg) => ("SearchError", msg),
        spec_db_core::SpecDbError::GraphError(msg) => ("GraphError", msg),
        spec_db_core::SpecDbError::SyncError(msg) => ("SyncError", msg),
        spec_db_core::SpecDbError::IngestError(msg) => ("IngestError", msg),
        spec_db_core::SpecDbError::ConsistencyError(msg) => ("ConsistencyError", msg),
        spec_db_core::SpecDbError::ConfigError(msg) => ("ConfigError", msg),
    };

    json!({
        "error_type": error_type,
        "message": message,
        "context": Value::Null,
    })
}

fn decode_custom_mcp_error(error: &spec_db_core::SpecDbError) -> Option<Value> {
    let raw = match error {
        spec_db_core::SpecDbError::SearchError(msg)
        | spec_db_core::SpecDbError::GraphError(msg)
        | spec_db_core::SpecDbError::SyncError(msg)
        | spec_db_core::SpecDbError::IngestError(msg)
        | spec_db_core::SpecDbError::ConsistencyError(msg)
        | spec_db_core::SpecDbError::ConfigError(msg) => msg,
    };

    let payload = raw.strip_prefix("mcp_error::")?;
    serde_json::from_str(payload).ok()
}
