use super::Modality;
use serde::{Deserialize, Serialize};

/// Metadata on the generation request's token usage.
///
/// 关于生成请求的令牌使用的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    /// Number of tokens in the prompt.
    ///
    /// 提示中的令牌数。
    #[serde(default)]
    pub prompt_token_count: i32,
    /// Number of tokens in the cached part of the prompt (the cached content).
    ///
    /// 提示的缓存部分（缓存内容）中的令牌数。
    #[serde(default)]
    pub cached_content_token_count: i32,
    /// Total number of tokens across all the generated response candidates.
    ///
    /// 所有生成的响应候选项的令牌总数。
    #[serde(default)]
    pub candidates_token_count: i32,
    /// Output only. Number of tokens present in tool-use prompt(s).
    ///
    /// 仅输出。工具使用提示中存在的令牌数。
    #[serde(default)]
    pub tool_use_prompt_token_count: i32,
    /// Output only. Number of tokens of thoughts for thinking models.
    ///
    /// 仅输出。思考模型的思考令牌数。
    #[serde(default)]
    pub thoughts_token_count: i32,
    /// 生成请求（提示 + 响应候选项）的总令牌数。
    #[serde(default)]
    pub total_token_count: i32,
    /// Output only. List of modalities that were processed in the request input.
    ///
    /// 仅输出。请求输入中处理的模态列表。
    #[serde(default)]
    pub prompt_tokens_details: Vec<ModalityTokenCount>,
    /// Output only. List of modalities of the cached content in the request input.
    ///
    /// 仅输出。请求输入中缓存内容的模态列表。
    #[serde(default)]
    pub cache_tokens_details: Vec<ModalityTokenCount>,
    /// Output only. List of modalities that were returned in the response.
    ///
    /// 仅输出。响应中返回的模态列表。
    #[serde(default)]
    pub candidates_tokens_details: Vec<ModalityTokenCount>,
    /// Output only. List of modalities that were processed for tool-use request inputs.
    ///
    /// 仅输出。为工具使用请求输入处理的模态列表。
    #[serde(default)]
    pub tool_use_prompt_tokens_details: Vec<ModalityTokenCount>,
    /// Output only. Service tier of the request.
    ///
    /// 仅输出。请求的服务层级。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
}

/// Service tier of the interaction.
///
/// 交互的服务层级。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceTier {
    ServiceTierUnspecified,
    ServiceTierFlex,
    ServiceTierStandard,
    ServiceTierPriority,
}

/// Represents token counting info for a single modality.
///
/// 表示单个模态的令牌计数信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    /// The modality associated with this token count.
    ///
    /// 与此令牌计数关联的模态。
    pub modality: Modality,
    /// Number of tokens.
    ///
    /// 令牌数量。
    pub token_count: i32,
}

/// Metadata on the usage of the cached content.
///
/// 以前存内容的使用的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedContentUsageMetadata {
    /// Total number of tokens that the cached content consumes.
    ///
    /// 缓存内容消耗的令牌总数。
    pub total_token_count: i32,
}
