use super::{CachedContentUsageMetadata, Content, Tool, ToolConfig};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

/// Cached content resource.
///
/// 缓存的内容资源。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedContent {
    /// Optional. Input only. Immutable. The content to cache.
    ///
    /// 可选。仅输入。不可变。要缓存的内容。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<Vec<Content>>,
    /// Optional. Input only. Immutable. A list of `Tools` the model may use to generate the next response.
    ///
    /// 可选。仅输入。不可变。模型可能用于生成下一个响应的 `Tools` 列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Optional. Timestamp in UTC of when this resource is considered expired.
    ///
    /// 可选。此资源被视为过期的 UTC 时间戳。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<StaticRefStr>,
    /// Optional. Input only. New TTL for this resource, input only.
    ///
    /// 可选。仅输入。此资源的新 TTL，仅输入。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<StaticRefStr>,
    /// Optional. Immutable. The user-generated meaningful display name of the cached content.
    ///
    /// 可选。不可变。用户生成的缓存内容有意义的显示名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<StaticRefStr>,
    /// Required. Immutable. The name of the `Model` to use for cached content.
    ///
    /// 必填。不可变。用于缓存内容的 `Model` 名称。
    pub model: StaticRefStr,
    /// Optional. Input only. Immutable. Developer set system instruction.
    ///
    /// 可选。仅输入。不可变。开发者设置的系统指令。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    /// Optional. Input only. Immutable. Tool config.
    ///
    /// 可选。仅输入。不可变。工具配置。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    /// Output only. The resource name referring to the content cache entry.
    ///
    /// 仅输出。引用内容缓存条目的资源名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<StaticRefStr>,
    /// Output only. Creation time of the cache.
    ///
    /// 仅输出。缓存的创建时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<StaticRefStr>,
    /// Output only. Update time of the cache.
    ///
    /// 仅输出。缓存的更新时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<StaticRefStr>,
    /// Output only. Metadata on the token usage.
    ///
    /// 仅输出。令牌使用的元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<CachedContentUsageMetadata>,
}
