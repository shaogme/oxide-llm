use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Specifies how the model should choose tool invocations.
///
/// 指定模型应如何选择工具调用。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ToolChoice {
    /// Automatically decide whether to use tools.
    ///
    /// 自动决定是否使用工具。
    Auto {
        /// Whether to disable parallel tool invocations.
        ///
        /// 是否禁用并行工具调用。
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },

    /// Force use of any provided tool.
    ///
    /// 强制使用任何可用工具。
    Any {
        /// Whether to disable parallel tool invocations.
        ///
        /// 是否禁用并行工具调用。
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },

    /// Force use of a specific named tool.
    ///
    /// 强制使用指定的工具。
    Tool {
        /// Name of the required tool.
        ///
        /// 要求的工具名称。
        name: StaticRefStr,

        /// Whether to disable parallel tool invocations.
        ///
        /// 是否禁用并行工具调用。
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },

    /// Disallow tool use.
    ///
    /// 禁止使用工具。
    #[serde(rename = "none")]
    None,
}

/// Tool definition parameter union.
///
/// 工具定义参数联合枚举。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tool {
    /// Custom user-defined schema tool.
    ///
    /// 自定义 Schema 工具。
    Custom(CustomTool),

    /// Pre-defined Bash tool.
    ///
    /// 预定义的 Bash 工具。
    Bash(BashTool),

    /// Pre-defined text editor tool.
    ///
    /// 预定义的文本编辑器工具。
    TextEditor(TextEditorTool),

    /// Pre-defined web search tool.
    ///
    /// 预定义的 Web 搜索工具。
    WebSearch(WebSearchTool),

    /// Pre-defined web fetch tool.
    ///
    /// 预定义的 Web 获取工具。
    WebFetch(WebFetchTool),

    /// Pre-defined code execution tool.
    ///
    /// 预定义的代码执行工具。
    CodeExecution(CodeExecutionTool),

    /// Pre-defined memory tool.
    ///
    /// 预定义的 Memory 工具。
    Memory(MemoryTool),

    /// Pre-defined tool search tool (BM25).
    ///
    /// 预定义的 BM25 工具搜索工具。
    ToolSearchBm25(ToolSearchToolBm25),

    /// Pre-defined tool search tool (Regex).
    ///
    /// 预定义的 Regex 工具搜索工具。
    ToolSearchRegex(ToolSearchToolRegex),
}

/// Custom tool definition with JSON schema input.
///
/// 带有 JSON Schema 输入格式的自定义工具定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    /// Tool name.
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Human-readable tool description.
    ///
    /// 可读的工具描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<StaticRefStr>,

    /// JSON schema defining accepted tool input parameters.
    ///
    /// 定义该工具输入参数的 JSON Schema。
    pub input_schema: Value,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,

    /// Optional type tag ("custom").
    ///
    /// 可选的类型标识（固定为 "custom"）。
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub r#type: Option<StaticRefStr>,

    /// Strict schema validation option.
    ///
    /// 是否开启严格格式校验。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,

    /// List of callers allowed to invoke this tool.
    ///
    /// 允许调用此工具的 Caller 列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<StaticRefStr>>,
}

/// Pre-defined Bash tool specification.
///
/// 预定义的 Bash 工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool {
    /// Tool name ("bash").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag (e.g. "bash_20250124").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,

    /// Strict schema validation option.
    ///
    /// 是否开启严格格式校验。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Pre-defined text editor tool specification.
///
/// 预定义的文本编辑器工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool {
    /// Tool name ("str_replace_editor" or "str_replace_based_edit_tool").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag (e.g. "text_editor_20250124").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,

    /// Strict schema validation option.
    ///
    /// 是否开启严格格式校验。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,

    /// Maximum number of characters to display when viewing a file.
    ///
    /// 查看文件时显示的最大字符数量。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,
}

/// Pre-defined Web Search tool specification.
///
/// 预定义的 Web 搜索工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    /// Tool name ("web_search").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag (e.g. "web_search_20250305").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Allowed domains list.
    ///
    /// 允许搜索的域名列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<StaticRefStr>>,

    /// Blocked domains list.
    ///
    /// 禁止搜索的域名列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<StaticRefStr>>,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,

    /// Maximum number of uses allowed.
    ///
    /// 允许的最大使用次数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,

    /// Strict schema validation option.
    ///
    /// 是否开启严格格式校验。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,

    /// User location context for local search relevance.
    ///
    /// 用于提高本地搜索相关性的用户位置信息。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
}

/// User location details for Web Search.
///
/// Web 搜索使用的用户位置详情。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchUserLocation {
    /// Location type tag ("approximate").
    ///
    /// 位置类型（固定为 "approximate"）。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// City name.
    ///
    /// 城市名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<StaticRefStr>,

    /// Country code/name.
    ///
    /// 国家代码或名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<StaticRefStr>,

    /// Region/state name.
    ///
    /// 地区或省份名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<StaticRefStr>,

    /// Timezone string.
    ///
    /// 时区字符串。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<StaticRefStr>,
}

/// Pre-defined Web Fetch tool specification.
///
/// 预定义的 Web 获取工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool {
    /// Tool name ("web_fetch").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag (e.g. "web_fetch_20250910").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Allowed domains list.
    ///
    /// 允许获取的域名列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<StaticRefStr>>,

    /// Blocked domains list.
    ///
    /// 禁止获取的域名列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<StaticRefStr>>,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,

    /// Maximum number of uses allowed.
    ///
    /// 允许的最大使用次数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
}

/// Pre-defined Code Execution tool specification.
///
/// 预定义的代码执行工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool {
    /// Tool name ("code_execution").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag (e.g. "code_execution_20250522").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,

    /// Cache control breakpoint.
    ///
    /// 缓存控制断点。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::content::CacheControl>,
}

/// Pre-defined Memory tool specification.
///
/// 预定义的 Memory 工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTool {
    /// Tool name ("memory").
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag ("memory_20250818").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}

/// Pre-defined BM25 Tool Search tool specification.
///
/// 预定义的 BM25 工具搜索工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolBm25 {
    /// Tool name.
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag ("tool_search_tool_bm25_20251119").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}

/// Pre-defined Regex Tool Search tool specification.
///
/// 预定义的 Regex 工具搜索工具规范。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolRegex {
    /// Tool name.
    ///
    /// 工具名称。
    pub name: StaticRefStr,

    /// Tool type tag ("tool_search_tool_regex_20251119").
    ///
    /// 工具类型标识。
    #[serde(rename = "type")]
    pub r#type: StaticRefStr,
}
