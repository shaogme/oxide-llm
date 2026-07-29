use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use super::{request::ThinkingSummaries, tool::Tool};

/// Agent resource for Gemini Interactions API.
///
/// Gemini Interactions API 中的 Agent 资源。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent.
    ///
    /// Agent 的唯一标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<StaticRefStr>,
    /// Agent description for developers.
    ///
    /// 面向开发者的 Agent 描述。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// System instruction for the agent.
    ///
    /// Agent 的系统指令。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,
    /// Base agent to extend.
    ///
    /// 扩展的基础 Agent。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_agent: Option<StaticRefStr>,
    /// Tools available to the agent.
    ///
    /// Agent 可用的工具。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Agent configuration.
    ///
    /// Agent 配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,
}

/// Agent configuration variants.
///
/// Agent 配置变体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentConfig {
    /// Antigravity agent config.
    ///
    /// Antigravity Agent 配置。
    #[serde(rename = "antigravity")]
    Antigravity(AntigravityAgentConfig),
    /// CodeMender agent config.
    ///
    /// CodeMender Agent 配置。
    #[serde(rename = "code-mender")]
    CodeMender(CodeMenderAgentConfig),
    /// Deep research agent config.
    ///
    /// 深度研究 Agent 配置。
    #[serde(rename = "deep-research")]
    DeepResearch(DeepResearchAgentConfig),
    /// Dynamic agent config.
    ///
    /// 动态 Agent 配置。
    #[serde(rename = "dynamic")]
    Dynamic(DynamicAgentConfig),
}

/// Configuration for Antigravity agent.
///
/// Antigravity Agent 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityAgentConfig {
    /// Model to use for agent reasoning.
    ///
    /// 用于 Agent 推理的模型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StaticRefStr>,
    /// Max total tokens for the agent run.
    ///
    /// Agent 运行的最大总 Token 数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_tokens: Option<StaticRefStr>,
}

/// Configuration for CodeMender agent.
///
/// CodeMender Agent 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMenderAgentConfig {
    /// Model to use.
    ///
    /// 使用的模型。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StaticRefStr>,
    /// Session ID for grouping.
    ///
    /// 用于分组的会话 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<StaticRefStr>,
    /// Find request parameters.
    ///
    /// 查找漏洞请求参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub find_request: Option<serde_json::Value>,
    /// Fix request parameters.
    ///
    /// 修复漏洞请求参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_request: Option<serde_json::Value>,
    /// Session configuration.
    ///
    /// 会话配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_config: Option<serde_json::Value>,
}

/// Configuration for Deep Research agent.
///
/// 深度研究 Agent 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchAgentConfig {
    /// Collaborative planning flag.
    ///
    /// 协同规划标志。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collaborative_planning: Option<bool>,
    /// Enable BigQuery tool flag.
    ///
    /// 启用 BigQuery 工具标志。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_bigquery_tool: Option<bool>,
    /// Thinking summaries configuration.
    ///
    /// 思考摘要配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_summaries: Option<ThinkingSummaries>,
    /// Visualization option.
    ///
    /// 可视化选项。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visualization: Option<StaticRefStr>,
}

/// Configuration for Dynamic agent.
///
/// 动态 Agent 配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicAgentConfig {}

/// Response for listing agents.
///
/// 列出 Agent 的响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsResponse {
    /// Array of agent definitions.
    ///
    /// Agent 定义列表。
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// Pagination token for next page.
    ///
    /// 下一页的分页 Token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}
