use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    agent::AgentConfig,
    content::Content,
    step::Step,
    tool::{Tool, ToolChoiceConfig},
    webhook::WebhookConfig,
};

/// Parameters for creating an interaction (Model or Agent).
///
/// 创建 Interaction 的参数（模型或 Agent）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInteractionRequest {
    /// Model name option (e.g. 'gemini-3.6-flash'). Required if agent is not set.
    ///
    /// 模型名称选项（如 'gemini-3.6-flash'）。若未设置 agent，则为必填。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StaticRefStr>,
    /// Agent name option (e.g. 'antigravity-preview-05-2026'). Required if model is not set.
    ///
    /// Agent 名称选项（如 'antigravity-preview-05-2026'）。若未设置 model，则为必填。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<StaticRefStr>,
    /// Required. Inputs for the interaction (String, Content, Contents, Steps, or Turns).
    ///
    /// 必填。Interaction 的输入（字符串、Content、Content 数组、Step 数组或 Turn 数组）。
    pub input: InteractionsInput,
    /// Developer system instruction.
    ///
    /// 开发者设置的系统指令。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,
    /// List of tools available during interaction.
    ///
    /// 交互期间可用的工具声明列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Enforces output JSON object schema or format.
    ///
    /// 强制生成的响应为 JSON 对象模式或格式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,
    /// Whether interaction response will be streamed.
    ///
    /// 是否流式传输 Interaction 响应。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Whether to store response for later retrieval.
    ///
    /// 是否存储响应以备日后检索。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// Whether to run in background.
    ///
    /// 是否在后台运行。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// Configuration for model interaction.
    ///
    /// 模型交互的配置参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    /// Configuration for agent interaction.
    ///
    /// Agent 交互的配置参数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,
    /// Environment configuration.
    ///
    /// 环境配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<serde_json::Value>,
    /// Labels with user defined metadata.
    ///
    /// 包含用户自定义元数据的标签。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    /// ID of previous interaction for multi-turn dialogue.
    ///
    /// 用于多轮对话的上一个 Interaction 的 ID。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,
    /// Safety settings for interaction.
    ///
    /// 交互的安全设置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    /// Service tier ('flex', 'standard', 'priority').
    ///
    /// 服务层级（'flex', 'standard', 'priority'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Webhook configuration.
    ///
    /// Webhook 配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_config: Option<WebhookConfig>,
}

/// Inputs for interaction.
///
/// Interaction 的输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InteractionsInput {
    /// Plain text string prompt.
    ///
    /// 纯文本提示词。
    String(String),
    /// Single content block.
    ///
    /// 单个内容块。
    Content(Content),
    /// List of content blocks.
    ///
    /// 内容块列表。
    Contents(Vec<Content>),
    /// List of steps.
    ///
    /// 步骤列表。
    Steps(Vec<Step>),
    /// List of turns.
    ///
    /// 对话轮次列表。
    Turns(Vec<Turn>),
}

/// A conversation turn.
///
/// 对话轮次。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Content of turn.
    ///
    /// 轮次内容。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<TurnContent>,
    /// Role of turn originator ('user' or 'model').
    ///
    /// 轮次发起者的角色（'user' 或 'model'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<StaticRefStr>,
}

/// Turn content variant (String or Contents).
///
/// 轮次内容变体（字符串或 Content 列表）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TurnContent {
    /// Text prompt.
    ///
    /// 文本提示词。
    String(String),
    /// Content blocks.
    ///
    /// 内容块列表。
    Contents(Vec<Content>),
}

/// Configuration parameters for model interactions.
///
/// 模型交互的配置参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    /// Maximum number of tokens in response.
    ///
    /// 响应中包含的最大 Token 数。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    /// Seed used in decoding.
    ///
    /// 解码中使用的种子。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    /// Speech generation config.
    ///
    /// 语音生成配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    /// Stop character sequences.
    ///
    /// 停止字符序列。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<StaticRefStr>>,
    /// Level of thought tokens generated.
    ///
    /// 模型应生成的思考 Token 级别。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
    /// Thought summaries config.
    ///
    /// 是否在响应中包含思考摘要。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_summaries: Option<ThinkingSummaries>,
    /// Tool choice config or mode string.
    ///
    /// 工具选择配置或模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Speech recognition configuration.
    ///
    /// 语音识别（转录）配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_config: Option<TranscriptionConfig>,
    /// Video generation configuration.
    ///
    /// 视频生成配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_config: Option<VideoConfig>,
}

/// Speech generation config.
///
/// 语音生成配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechConfig {
    /// Speech language code.
    ///
    /// 语音语言代码。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<StaticRefStr>,
    /// Speaker name.
    ///
    /// 说话人名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<StaticRefStr>,
    /// Speaker voice.
    ///
    /// 说话人声音。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<StaticRefStr>,
}

/// Transcription config for ASR.
///
/// 语音识别（ASR）转录配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    /// BCP-47 language codes hints.
    ///
    /// BCP-47 语言代码提示。
    pub language_hints: Vec<StaticRefStr>,
    /// Custom vocabulary phrases.
    ///
    /// 自定义词汇短语列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_vocabulary: Option<Vec<String>>,
    /// Diarization mode ('speaker').
    ///
    /// 说话人日志（分化）模式。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diarization_mode: Option<StaticRefStr>,
    /// Timestamp granularities ('word').
    ///
    /// 时间戳粒度（如 'word'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_granularities: Option<Vec<StaticRefStr>>,
}

/// Video generation config.
///
/// 视频生成配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Task mode ('text_to_video', 'image_to_video', 'reference_to_video', 'edit').
    ///
    /// 任务模式（'text_to_video', 'image_to_video', 'reference_to_video', 'edit'）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<StaticRefStr>,
}

/// Tool choice configuration or enum value.
///
/// 工具选择配置或枚举值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Mode enum string ('auto', 'any', 'none', 'validated').
    ///
    /// 模式字符串（'auto', 'any', 'none', 'validated'）。
    Mode(StaticRefStr),
    /// Detailed tool choice configuration.
    ///
    /// 详细的工具选择配置。
    Config(ToolChoiceConfig),
}

/// Level of thought tokens generated.
///
/// 思考 Token 级别。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingLevel {
    Minimal,
    Low,
    Medium,
    High,
}

/// Thought summaries setting.
///
/// 思考摘要设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingSummaries {
    Auto,
    None,
}

/// Safety setting for interactions.
///
/// 交互的安全设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    /// Harm category.
    ///
    /// 伤害类别。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<StaticRefStr>,
    /// Harm block threshold.
    ///
    /// 伤害阻止阈值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<StaticRefStr>,
    /// Harm block method.
    ///
    /// 伤害阻止方法。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<StaticRefStr>,
}

/// Service tier for interaction.
///
/// 交互的服务层级。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Flex,
    Standard,
    Priority,
}
