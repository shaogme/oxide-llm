use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Webhook configuration message for an interaction request.
///
/// 用于 Interaction 请求的 Webhook 配置消息。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookConfig {
    /// Webhook URIs override.
    ///
    /// 覆盖使用的 Webhook URI 列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uris: Option<Vec<String>>,
    /// User metadata returned on each event emission.
    ///
    /// 每个事件发送时返回的用户元数据。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Webhook resource object.
///
/// Webhook 资源对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Identifier for webhook.
    ///
    /// Webhook 标识符。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Webhook endpoint URI.
    ///
    /// Webhook 端点 URI。
    pub uri: String,
    /// Subscribed events list.
    ///
    /// 订阅事件列表。
    pub subscribed_events: Vec<StaticRefStr>,
    /// User-provided name of webhook.
    ///
    /// 用户提供的 Webhook 名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// State of webhook ('enabled', 'disabled', 'disabled_due_to_failed_deliveries').
    ///
    /// Webhook 状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WebhookState>,
    /// Signing secrets.
    ///
    /// 签名密钥。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secrets: Option<Vec<SigningSecret>>,
    /// Creation timestamp.
    ///
    /// 创建时间戳。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// Last update timestamp.
    ///
    /// 最近更新时间戳。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

/// State of webhook.
///
/// Webhook 的状态。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookState {
    Enabled,
    Disabled,
    DisabledDueToFailedDeliveries,
}

/// Webhook update fields.
///
/// Webhook 更新字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookUpdate {
    /// User-provided name.
    ///
    /// 用户提供的名称。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Webhook URI.
    ///
    /// Webhook 端点 URI。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Subscribed events list.
    ///
    /// 订阅事件列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_events: Option<Vec<StaticRefStr>>,
    /// Webhook state.
    ///
    /// Webhook 状态。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WebhookState>,
}

/// Response for listing webhooks.
///
/// 列出 Webhook 的响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListWebhooksResponse {
    /// List of webhooks.
    ///
    /// Webhook 列表。
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
    /// Next page token.
    ///
    /// 下一页 Token。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Ping webhook request.
///
/// Ping Webhook 请求。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PingWebhookRequest {}

/// Ping webhook response.
///
/// Ping Webhook 响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PingWebhookResponse {}

/// Rotate signing secret request.
///
/// 轮换签名密钥请求。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotateSigningSecretRequest {
    /// Revocation behavior ('revoke_previous_secrets_after_h24' or 'revoke_previous_secrets_immediately').
    ///
    /// 撤销行为。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_behavior: Option<StaticRefStr>,
}

/// Rotate signing secret response.
///
/// 轮换签名密钥响应。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotateSigningSecretResponse {
    /// Newly generated signing secret.
    ///
    /// 新生成的签名密钥。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

/// Signing secret details.
///
/// 签名密钥详情。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SigningSecret {
    /// Truncated secret.
    ///
    /// 截断后的密钥。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated_secret: Option<String>,
    /// Expiration date of signing secret.
    ///
    /// 签名密钥的到期时间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<String>,
}
