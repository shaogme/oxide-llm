use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::{content::Content, response::Status};

/// A step in the interaction.
///
/// Interaction 中的步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Step {
    /// User input step.
    ///
    /// 用户输入步骤。
    #[serde(rename = "user_input")]
    UserInput(UserInputStep),
    /// Model output step.
    ///
    /// 模型输出步骤。
    #[serde(rename = "model_output")]
    ModelOutput(ModelOutputStep),
    /// Thought step.
    ///
    /// 思考步骤。
    #[serde(rename = "thought")]
    Thought(ThoughtStep),
    /// Function call step.
    ///
    /// 函数调用步骤。
    #[serde(rename = "function_call")]
    FunctionCall(FunctionCallStep),
    /// Function result step.
    ///
    /// 函数结果步骤。
    #[serde(rename = "function_result")]
    FunctionResult(FunctionResultStep),
    /// Code execution call step.
    ///
    /// 代码执行调用步骤。
    #[serde(rename = "code_execution_call")]
    CodeExecutionCall(CodeExecutionCallStep),
    /// Code execution result step.
    ///
    /// 代码执行结果步骤。
    #[serde(rename = "code_execution_result")]
    CodeExecutionResult(CodeExecutionResultStep),
    /// Google search call step.
    ///
    /// Google 搜索调用步骤。
    #[serde(rename = "google_search_call")]
    GoogleSearchCall(GoogleSearchCallStep),
    /// Google search result step.
    ///
    /// Google 搜索结果步骤。
    #[serde(rename = "google_search_result")]
    GoogleSearchResult(GoogleSearchResultStep),
    /// Google maps call step.
    ///
    /// Google 地图调用步骤。
    #[serde(rename = "google_maps_call")]
    GoogleMapsCall(GoogleMapsCallStep),
    /// Google maps result step.
    ///
    /// Google 地图结果步骤。
    #[serde(rename = "google_maps_result")]
    GoogleMapsResult(GoogleMapsResultStep),
    /// URL context call step.
    ///
    /// URL 上下文调用步骤。
    #[serde(rename = "url_context_call")]
    UrlContextCall(UrlContextCallStep),
    /// URL context result step.
    ///
    /// URL 上下文结果步骤。
    #[serde(rename = "url_context_result")]
    UrlContextResult(UrlContextResultStep),
    /// File search call step.
    ///
    /// 文件搜索调用步骤。
    #[serde(rename = "file_search_call")]
    FileSearchCall(FileSearchCallStep),
    /// File search result step.
    ///
    /// 文件搜索结果步骤。
    #[serde(rename = "file_search_result")]
    FileSearchResult(FileSearchResultStep),
    /// MCP server tool call step.
    ///
    /// MCP 服务器工具调用步骤。
    #[serde(rename = "mcp_server_tool_call")]
    McpServerToolCall(McpServerToolCallStep),
    /// MCP server tool result step.
    ///
    /// MCP 服务器工具结果步骤。
    #[serde(rename = "mcp_server_tool_result")]
    McpServerToolResult(McpServerToolResultStep),
}

/// User input step.
///
/// 用户输入步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputStep {
    /// Content of the user input.
    ///
    /// 用户输入的内容。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Content>>,
}

/// Model output step.
///
/// 模型输出步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutputStep {
    /// Generated content from model.
    ///
    /// 模型生成的输出内容。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Content>>,
    /// Error status if output failed.
    ///
    /// 失败时的错误状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Status>,
}

/// Thought step.
///
/// 思考步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStep {
    /// Signature hash for validation.
    ///
    /// 后端验证的签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
    /// Summary of thoughts.
    ///
    /// 思考摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Vec<Content>>,
}

/// Function call step.
///
/// 函数调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Function name.
    ///
    /// 函数名称。
    pub name: StaticRefStr,
    /// Function call arguments.
    ///
    /// 函数调用参数。
    pub arguments: serde_json::Value,
}

/// Function result step.
///
/// 函数结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResultStep {
    /// Call ID corresponding to FunctionCallStep.
    ///
    /// 对应的函数调用 ID。
    pub call_id: StaticRefStr,
    /// Name of tool called.
    ///
    /// 调用的工具名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// Function call result output.
    ///
    /// 函数调用的结果输出。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Whether tool call resulted in an error.
    ///
    /// 工具调用是否产生错误。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Code execution call step.
///
/// 代码执行调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Arguments for code execution (code, language).
    ///
    /// 代码执行的参数（代码，语言）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Signature hash for validation.
    ///
    /// 验证签名。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Code execution result step.
///
/// 代码执行结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionResultStep {
    /// Call ID corresponding to CodeExecutionCallStep.
    ///
    /// 对应的代码执行调用 ID。
    pub call_id: StaticRefStr,
    /// Execution output text.
    ///
    /// 执行输出文本。
    pub result: String,
    /// Whether execution errored.
    ///
    /// 执行是否报错。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Google search call step.
///
/// Google 搜索调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSearchCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Search arguments.
    ///
    /// 搜索参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Type of search grounding enabled.
    ///
    /// 启用的搜索依据类型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<StaticRefStr>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Google search result step.
///
/// Google 搜索结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSearchResultStep {
    /// Call ID corresponding to GoogleSearchCallStep.
    ///
    /// 对应的 Google 搜索调用 ID。
    pub call_id: StaticRefStr,
    /// Search result items.
    ///
    /// 搜索结果列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Vec<serde_json::Value>>,
    /// Whether search errored.
    ///
    /// 搜索是否报错。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Google maps call step.
///
/// Google 地图调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleMapsCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Google maps call arguments.
    ///
    /// Google 地图调用参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// Google maps result step.
///
/// Google 地图结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleMapsResultStep {
    /// Call ID corresponding to GoogleMapsCallStep.
    ///
    /// 对应的 Google 地图调用 ID。
    pub call_id: StaticRefStr,
    /// Maps result items.
    ///
    /// 地图结果列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Vec<serde_json::Value>>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// URL context call step.
///
/// URL 上下文调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlContextCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// URL context arguments (URLs to retrieve).
    ///
    /// URL 上下文参数（要检索的 URL）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// URL context result step.
///
/// URL 上下文结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlContextResultStep {
    /// Call ID corresponding to UrlContextCallStep.
    ///
    /// 对应的 URL 上下文调用 ID。
    pub call_id: StaticRefStr,
    /// URL context retrieval result list.
    ///
    /// URL 上下文检索结果列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Vec<serde_json::Value>>,
    /// Whether URL retrieval errored.
    ///
    /// URL 检索是否报错。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// File search call step.
///
/// 文件搜索调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// File search result step.
///
/// 文件搜索结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResultStep {
    /// Call ID corresponding to FileSearchCallStep.
    ///
    /// 对应的文件搜索调用 ID。
    pub call_id: StaticRefStr,
    /// Signature hash.
    ///
    /// 签名哈希。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<StaticRefStr>,
}

/// MCP server tool call step.
///
/// MCP 服务器工具调用步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerToolCallStep {
    /// Unique ID for tool call.
    ///
    /// 工具调用的唯一标识 ID。
    pub id: StaticRefStr,
    /// Name of tool called.
    ///
    /// 调用的工具名称。
    pub name: StaticRefStr,
    /// Server name of MCP server.
    ///
    /// MCP 服务器的名称。
    pub server_name: StaticRefStr,
    /// Arguments passed to MCP tool.
    ///
    /// 传递给 MCP 工具的参数。
    pub arguments: serde_json::Value,
}

/// MCP server tool result step.
///
/// MCP 服务器工具结果步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerToolResultStep {
    /// Call ID corresponding to McpServerToolCallStep.
    ///
    /// 对应的 MCP 工具调用 ID。
    pub call_id: StaticRefStr,
    /// Tool name.
    ///
    /// 工具名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// Server name.
    ///
    /// 服务器名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<StaticRefStr>,
    /// MCP tool call result.
    ///
    /// MCP 工具调用结果。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}
