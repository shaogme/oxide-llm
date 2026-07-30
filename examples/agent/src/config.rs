use serde::Deserialize;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Secret string wrapper that redacts sensitive content in debug/display formatting.
///
/// 敏感字符串包装器，在 debug/display 格式化时隐藏敏感内容。
#[derive(Deserialize, Clone)]
pub struct SecretString(String);

impl Debug for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "********")
    }
}

impl Display for SecretString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "********")
    }
}

impl SecretString {
    /// Returns the underlying secret string slice.
    ///
    /// 返回底层敏感字符串切片。
    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// Provider configuration type enum.
///
/// 供应商配置类型枚举。
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// OpenAI API provider.
    ///
    /// OpenAI API 供应商。
    OpenAI,
    /// Claude API provider.
    ///
    /// Claude API 供应商。
    Claude,
    /// Gemini API provider.
    ///
    /// Gemini API 供应商。
    Gemini,
}

/// Configuration structure for a provider.
///
/// 供应商配置结构体。
#[derive(Deserialize, Clone, Debug)]
pub struct ProviderConfig {
    /// Unique provider identifier.
    ///
    /// 供应商唯一标识符。
    pub id: String,
    /// Type of provider API.
    ///
    /// 供应商 API 类型。
    pub r#type: ProviderType,
    /// Base URL for API requests.
    ///
    /// API 请求的基础 URL。
    pub base_url: String,
    /// Authentication API key.
    ///
    /// 身份验证 API 密钥。
    pub api_key: SecretString,
}

/// Configuration structure for a model.
///
/// 模型配置结构体。
#[derive(Deserialize, Clone, Debug)]
pub struct ModelConfig {
    /// Name identifier of this model configuration entry.
    ///
    /// 此模型配置条目的名称标识符。
    pub name: String,
    /// Associated provider ID.
    ///
    /// 关联的供应商 ID。
    pub provider: String,
    /// Actual API model name used in requests.
    ///
    /// 请求中使用的实际 API 模型名称。
    pub model: String,
    /// API endpoint specification.
    ///
    /// API 端点规范。
    pub endpoint: String,
    /// Maximum tokens limit for response generation (optional).
    ///
    /// 响应生成的最大 token 限制（可选）。
    pub max_tokens: Option<u32>,
}

/// Root configuration structure.
///
/// 根配置结构体。
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Name of the currently active model configuration.
    ///
    /// 当前激活的模型配置名称。
    pub active_model: Option<String>,
    /// List of provider configurations.
    ///
    /// 供应商配置列表。
    pub providers: Vec<ProviderConfig>,
    /// List of model configurations.
    ///
    /// 模型配置列表。
    pub models: Vec<ModelConfig>,
}

impl Config {
    /// Finds a model configuration by name along with its associated provider.
    ///
    /// 根据名称查找模型配置及其关联的供应商配置。
    pub fn find_model_and_provider(
        &self,
        name: &str,
    ) -> Result<(&ModelConfig, &ProviderConfig), String> {
        let model = self
            .models
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format!("Model configuration with name '{}' not found", name))?;

        let provider = self
            .providers
            .iter()
            .find(|p| p.id == model.provider)
            .ok_or_else(|| {
                format!(
                    "Provider '{}' referenced by model '{}' not found",
                    model.provider, name
                )
            })?;

        Ok((model, provider))
    }

    /// Selects a model configuration and provider using explicit name or active_model setting.
    ///
    /// 使用显式名称或 active_model 设置选择模型配置与供应商。
    pub fn select_model_and_provider(
        &self,
        explicit_name: Option<&str>,
    ) -> Result<(&ModelConfig, &ProviderConfig), String> {
        let name = match explicit_name {
            Some(n) => n,
            None => self.active_model.as_deref().ok_or_else(|| {
                "No active model specified in configuration or CLI arguments".to_string()
            })?,
        };

        self.find_model_and_provider(name)
    }
}
