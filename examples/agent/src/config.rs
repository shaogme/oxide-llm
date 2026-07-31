use oxide_llm::Config as OxideConfig;
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

/// Model configuration type enum.
///
/// 模型配置类型枚举。
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelType {
    /// OpenAI Responses API model.
    ///
    /// OpenAI Responses API 模型。
    #[serde(rename = "openai-responses", alias = "openai_responses")]
    OpenAiResponses,
    /// OpenAI Chat Completions API model.
    ///
    /// OpenAI Chat Completions API 模型。
    #[serde(rename = "openai-chat-completions", alias = "openai_chat_completions")]
    OpenAiChatCompletions,
    /// Claude Messages API model.
    ///
    /// Claude Messages API 模型。
    #[serde(rename = "claude-messages", alias = "claude_messages", alias = "claude")]
    ClaudeMessages,
    /// Gemini Interactions API model.
    ///
    /// Gemini Interactions API 模型。
    #[serde(rename = "gemini-interactions", alias = "gemini_interactions")]
    GeminiInteractions,
    /// Gemini Generate Content API model.
    ///
    /// Gemini Generate Content API 模型。
    #[serde(rename = "gemini-generate-content", alias = "gemini_generate_content")]
    GeminiGenerateContent,
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
    /// Type of model API.
    ///
    /// 模型 API 类型。
    pub r#type: ModelType,
    /// Unified LLM agent configuration.
    ///
    /// 统一的 LLM 代理配置。
    #[serde(flatten)]
    pub config: OxideConfig,
}

impl ModelConfig {
    /// Returns the model name or empty string if unset.
    ///
    /// 返回模型名称，若未设置则返回空字符串。
    pub fn model(&self) -> &str {
        self.config.model()
    }

    /// Returns the API endpoint or empty string if unset.
    ///
    /// 返回 API 端点，若未设置则返回空字符串。
    pub fn endpoint(&self) -> &str {
        self.config.endpoint().unwrap_or_default()
    }

    /// Returns reference to unified `oxide_llm::Config`.
    ///
    /// 返回统一 `oxide_llm::Config` 的引用。
    pub fn oxide_config(&self) -> &OxideConfig {
        &self.config
    }

    /// Converts this model configuration into unified `oxide_llm::Config`.
    ///
    /// 将此模型配置转换为统一的 `oxide_llm::Config`。
    pub fn to_oxide_config(&self) -> OxideConfig {
        self.config.clone()
    }
}

impl From<&ModelConfig> for OxideConfig {
    fn from(config: &ModelConfig) -> Self {
        config.config.clone()
    }
}

impl From<ModelConfig> for OxideConfig {
    fn from(config: ModelConfig) -> Self {
        config.config
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_example_config() {
        let example = include_str!("../agent.toml.example");
        let config: Config = toml::from_str(example).unwrap();
        assert_eq!(config.active_model.as_deref(), Some("my-openai-responses"));
        assert_eq!(config.providers.len(), 3);
        assert_eq!(config.models.len(), 5);

        let (model, _) = config
            .find_model_and_provider("my-openai-responses")
            .unwrap();
        assert_eq!(model.r#type, ModelType::OpenAiResponses);
        assert_eq!(model.model(), "gpt-4.5-preview");
        assert_eq!(model.endpoint(), "responses");
        assert_eq!(model.config.temperature(), Some(0.7));
        assert_eq!(
            model.config.reasoning_effort(),
            Some(oxide_llm::config::ReasoningEffort::Medium)
        );
    }
}
