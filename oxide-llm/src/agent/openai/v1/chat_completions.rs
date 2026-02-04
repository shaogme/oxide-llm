use crate::transport::Transport;
use error_set::error_set;
use oxide_llm_core::mapper::MapperError;
use oxide_llm_core::message::{Message, MessageHistory};
use oxide_llm_proto::openai::v1::chat_completions::request::{
    AudioOptions, ChatCompletionMessage, ChatCompletionRequest, PredictionContent, ResponseFormat,
    Stop, StreamOptions, WebSearchOptions,
};
use oxide_llm_proto::openai::v1::chat_completions::response::ChatCompletionResponse;
use oxide_llm_proto::openai::v1::{FunctionDefinition, Tool, ToolChoice};
use std::collections::HashMap;

error_set! {
    ChatCompletionsError := {
        #[display("Transport error: {0}")]
        Transport(crate::transport::TransportError),
        #[display("Mapper conversion error: {0}")]
        Mapper(MapperError),
    }
}

/// Configuration for OpenAI Chat Completions Agent.
///
/// OpenAI Chat Completions 代理配置。
/// 包含了除 `messages` 和 `model` 之外的所有 `ChatCompletionRequest` 可选参数。
#[derive(Debug, Clone, Default)]
pub struct ChatCompletionsConfig {
    pub frequency_penalty: Option<f32>,
    pub logit_bias: Option<HashMap<String, f32>>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u8>,
    pub max_tokens: Option<u32>,
    pub max_completion_tokens: Option<u32>,
    pub n: Option<u8>,
    pub modalities: Option<Vec<String>>,
    pub prediction: Option<PredictionContent>,
    pub audio: Option<AudioOptions>,
    pub presence_penalty: Option<f32>,
    pub response_format: Option<ResponseFormat>,
    pub seed: Option<i64>,
    pub service_tier: Option<String>,
    pub stop: Option<Stop>,
    pub store: Option<bool>,
    pub stream: Option<bool>,
    pub stream_options: Option<StreamOptions>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub user: Option<String>,
    pub function_call: Option<serde_json::Value>,
    pub functions: Option<Vec<FunctionDefinition>>,
    pub web_search_options: Option<WebSearchOptions>,
    pub verbosity: Option<String>,
    pub reasoning_effort: Option<String>,
}

impl ChatCompletionsConfig {
    /// Convert Config to ChatCompletionRequest with provided messages and model.
    ///
    /// 将配置转换为 ChatCompletionRequest，并填入消息和模型。
    pub fn to_request(
        self,
        messages: Vec<ChatCompletionMessage>,
        model: String,
    ) -> ChatCompletionRequest {
        ChatCompletionRequest {
            messages,
            model,
            frequency_penalty: self.frequency_penalty,
            logit_bias: self.logit_bias,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            max_tokens: self.max_tokens,
            max_completion_tokens: self.max_completion_tokens,
            n: self.n,
            modalities: self.modalities,
            prediction: self.prediction,
            audio: self.audio,
            presence_penalty: self.presence_penalty,
            response_format: self.response_format,
            seed: self.seed,
            service_tier: self.service_tier,
            stop: self.stop,
            store: self.store,
            stream: self.stream,
            stream_options: self.stream_options,
            temperature: self.temperature,
            top_p: self.top_p,
            tools: self.tools,
            tool_choice: self.tool_choice,
            parallel_tool_calls: self.parallel_tool_calls,
            user: self.user,
            function_call: self.function_call,
            functions: self.functions,
            web_search_options: self.web_search_options,
            verbosity: self.verbosity,
            reasoning_effort: self.reasoning_effort,
        }
    }
}

/// OpenAI Chat Completions Agent.
///
/// OpenAI Chat Completions 代理。
/// 负责处理与 OpenAI Chat API 的基本交互，无需维护状态。
#[derive(Clone)]
pub struct ChatCompletionsAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The model name to use (e.g., "gpt-4o").
    ///
    /// 使用的模型名称 (例如 "gpt-4o")。
    model: String,
    /// The configuration for the chat completions request.
    ///
    /// 聊天补全请求的配置。
    config: ChatCompletionsConfig,
}

impl<T: Transport> ChatCompletionsAgent<T> {
    /// Create a new ChatCompletionsAgent.
    ///
    /// 创建一个新的 ChatCompletionsAgent。
    pub fn new(transport: T, model: String) -> Self {
        Self {
            transport,
            model,
            config: ChatCompletionsConfig::default(),
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: ChatCompletionsConfig) -> Self {
        self.config = config;
        self
    }

    /// Send a chat request to OpenAI.
    ///
    /// 发送聊天请求到 OpenAI。
    pub async fn chat(&self, history: MessageHistory) -> Result<Message, ChatCompletionsError> {
        let MessageHistory {
            system_prompt,
            messages,
        } = history;

        // 1. Convert Core Messages to OpenAI Messages
        // 1. 将核心消息转换为 OpenAI 消息
        let mut openai_messages: Vec<ChatCompletionMessage> = messages
            .into_iter()
            .map(|msg| msg.try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(ChatCompletionsError::Mapper)?;

        if let Some(prompt) = system_prompt {
            openai_messages.insert(
                0,
                ChatCompletionMessage::System {
                    content: prompt,
                    name: None,
                },
            );
        }

        // 2. Construct Request using Config
        // 2. 使用 Config 构建请求
        let request = self
            .config
            .clone()
            .to_request(openai_messages, self.model.clone());

        // 3. Send Request
        // 3. 发送请求
        let response: ChatCompletionResponse = self
            .transport
            .send("/v1/chat/completions", request)
            .await
            .map_err(ChatCompletionsError::Transport)?;

        // 4. Convert Response back to Core Message
        // 4. 将响应转换回核心消息
        let core_message: Message = response.try_into().map_err(ChatCompletionsError::Mapper)?;

        Ok(core_message)
    }
}
