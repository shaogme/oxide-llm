use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool declaration for Gemini Interactions API.
///
/// Gemini Interactions API 中的工具声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    /// Function tool declaration.
    ///
    /// 函数工具声明。
    #[serde(rename = "function")]
    Function(FunctionTool),
    /// Code execution tool declaration.
    ///
    /// 代码执行工具声明。
    #[serde(rename = "code_execution")]
    CodeExecution(CodeExecutionTool),
    /// Google search tool declaration.
    ///
    /// Google 搜索工具声明。
    #[serde(rename = "google_search")]
    GoogleSearch(GoogleSearchTool),
    /// Google maps tool declaration.
    ///
    /// Google 地图工具声明。
    #[serde(rename = "google_maps")]
    GoogleMaps(GoogleMapsTool),
    /// URL context retrieval tool declaration.
    ///
    /// URL 上下文检索工具声明。
    #[serde(rename = "url_context")]
    UrlContext(UrlContextTool),
    /// File search tool declaration.
    ///
    /// 文件搜索工具声明。
    #[serde(rename = "file_search")]
    FileSearch(FileSearchTool),
    /// MCP server tool declaration.
    ///
    /// MCP 服务器工具声明。
    #[serde(rename = "mcp_server")]
    McpServer(McpServerTool),
    /// Computer use tool declaration.
    ///
    /// 计算机使用工具声明。
    #[serde(rename = "computer_use")]
    ComputerUse(ComputerUseTool),
    /// File retrieval tool declaration.
    ///
    /// 文件检索工具声明。
    #[serde(rename = "retrieval")]
    Retrieval(RetrievalTool),
}

/// Function tool declaration.
///
/// 函数工具声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    /// The name of the function.
    ///
    /// 函数名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// A brief description of the function.
    ///
    /// 函数的简要描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,
    /// Open-API schema parameters of the function.
    ///
    /// 函数的 OpenAPI 模式参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Code execution tool.
///
/// 代码执行工具。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeExecutionTool {}

/// Google search tool.
///
/// Google 搜索工具。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleSearchTool {}

/// Google maps tool.
///
/// Google 地图工具。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleMapsTool {
    /// Optional latitude coordinate.
    ///
    /// 可选纬度坐标。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Optional longitude coordinate.
    ///
    /// 可选经度坐标。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

/// URL context retrieval tool.
///
/// URL 上下文检索工具。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UrlContextTool {}

/// File search tool.
///
/// 文件搜索工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchTool {
    /// File search store names.
    ///
    /// 文件搜索存储库名称列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search_store_names: Option<Vec<StaticRefStr>>,
    /// Metadata filter.
    ///
    /// 元数据过滤器。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<StaticRefStr>,
    /// Top K chunks to retrieve.
    ///
    /// 检索的前 K 个块。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}

/// MCP server tool declaration.
///
/// MCP 服务器工具声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerTool {
    /// The name of the MCP server.
    ///
    /// MCP 服务器的名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// Full URL endpoint of MCP server.
    ///
    /// MCP 服务器的完整 URL 端点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<StaticRefStr>,
    /// Authentication headers or optional headers.
    ///
    /// 身份验证请求头或可选请求头。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Allowed tools configuration.
    ///
    /// 允许的工具配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<AllowedTools>>,
}

/// Computer use tool declaration.
///
/// 计算机使用工具声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseTool {
    /// Operating environment ('browser', 'mobile', 'desktop').
    ///
    /// 操作环境（'browser', 'mobile', 'desktop'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<StaticRefStr>,
    /// Enable prompt injection detection check.
    ///
    /// 是否开启提示注入检测检查。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_prompt_injection_detection: Option<bool>,
    /// Excluded predefined functions.
    ///
    /// 排除的预定义函数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_predefined_functions: Option<Vec<StaticRefStr>>,
    /// Disabled safety policies.
    ///
    /// 禁用的安全策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_safety_policies: Option<Vec<StaticRefStr>>,
}

/// Retrieval tool declaration.
///
/// 检索工具声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalTool {
    /// Enabled retrieval types.
    ///
    /// 启用的检索类型列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_types: Option<Vec<StaticRefStr>>,
    /// Configuration for Vertex AI Search.
    ///
    /// Vertex AI Search 的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertex_ai_search_config: Option<serde_json::Value>,
    /// Configuration for RAG Store.
    ///
    /// RAG Store 的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_store_config: Option<serde_json::Value>,
    /// Configuration for Exa AI Search.
    ///
    /// Exa AI Search 的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exa_ai_search_config: Option<serde_json::Value>,
    /// Configuration for Parallel AI Search.
    ///
    /// Parallel AI Search 的配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_ai_search_config: Option<serde_json::Value>,
}

/// Allowed tools configuration.
///
/// 允许的工具配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedTools {
    /// Tool choice mode ('auto', 'any', 'none', 'validated').
    ///
    /// 工具选择模式（'auto', 'any', 'none', 'validated'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ToolChoiceMode>,
    /// Tool names allowed.
    ///
    /// 允许的工具名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<StaticRefStr>>,
}

/// Tool choice configuration containing allowed tools.
///
/// 包含允许工具的工具选择配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceConfig {
    /// Allowed tools.
    ///
    /// 允许的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<AllowedTools>,
}

/// Mode of tool choice.
///
/// 工具选择模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceMode {
    Auto,
    Any,
    None,
    Validated,
}
