use oxide_llm_core::mapper::openai::v1::OpenAIResponseMapper;
use oxide_llm_core::message::Message;
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::tool::{ToolAdapter, ToolChoiceAdapter};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::response::request::{CreateResponseRequest, InputItem, InputParam};
use oxide_llm_proto::openai::v1::response::response::Response;
use oxide_llm_proto::openai::v1::{Tool, ToolChoice};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::ChatAgent;
use crate::error::{AgentError, Result};

/// Configuration for OpenAI Responses Agent (Required).
///
/// OpenAI Response 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct ResponsesRequiredConfig {
    pub model: Cow<'static, str>,
    pub endpoint: Cow<'static, str>,
}

/// Configuration for OpenAI Responses Agent (Optional).
///
/// OpenAI Response 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct ResponsesOptionalConfig {
    pub include: Option<Vec<Cow<'static, str>>>,
    pub parallel_tool_calls: Option<bool>,
    pub store: Option<bool>,
    pub instructions: Option<Cow<'static, str>>,
    pub metadata: Option<HashMap<Cow<'static, str>, serde_json::Value>>,
    pub top_logprobs: Option<u8>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub user: Option<Cow<'static, str>>,
    pub safety_identifier: Option<Cow<'static, str>>,
    pub prompt_cache_key: Option<Cow<'static, str>>,
    pub service_tier: Option<Cow<'static, str>>,
    pub prompt_cache_retention: Option<Cow<'static, str>>,
    pub max_output_tokens: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub previous_response_id: Option<Cow<'static, str>>,
    pub background: Option<bool>,
}

/// Configuration for OpenAI Responses Agent.
///
/// OpenAI Response 代理配置。
#[derive(Debug, Clone)]
pub struct ResponsesConfig {
    pub required: ResponsesRequiredConfig,
    pub optional: ResponsesOptionalConfig,
}

impl ResponsesConfig {
    /// Convert Config to CreateResponseRequest with provided input.
    ///
    /// 将配置转换为 CreateResponseRequest，并填入输入参数。
    pub fn to_request(
        self,
        input: InputParam,
        tools: Option<Vec<Tool>>,
        tool_choice: Option<ToolChoice>,
        stream: Option<bool>,
    ) -> CreateResponseRequest {
        CreateResponseRequest {
            input,
            model: Some(self.required.model),
            include: self.optional.include,
            parallel_tool_calls: self.optional.parallel_tool_calls,
            store: self.optional.store,
            instructions: self.optional.instructions,
            stream,
            stream_options: None,
            conversation: None,
            metadata: self.optional.metadata,
            top_logprobs: self.optional.top_logprobs,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            tools,
            tool_choice,
            user: self.optional.user,
            safety_identifier: self.optional.safety_identifier,
            prompt_cache_key: self.optional.prompt_cache_key,
            service_tier: self.optional.service_tier,
            prompt_cache_retention: self.optional.prompt_cache_retention,
            max_output_tokens: self.optional.max_output_tokens,
            max_tool_calls: self.optional.max_tool_calls,
            previous_response_id: self.optional.previous_response_id,
            reasoning: None,
            background: self.optional.background,
            text: None,
            prompt: None,
            truncation: None,
        }
    }
}

/// OpenAI Responses Agent.
///
/// OpenAI Responses 代理。
/// 负责处理与 OpenAI Response API 的交互。
#[derive(Clone)]
pub struct ResponsesAgent<T: Clone> {
    transport: T,
    config: ResponsesConfig,
}

impl<T: Transport> ResponsesAgent<T> {
    /// Create a new ResponsesAgent.
    ///
    /// 创建一个新的 ResponsesAgent。
    pub fn new(transport: T, required: ResponsesRequiredConfig) -> Self {
        Self {
            transport,
            config: ResponsesConfig {
                required,
                optional: ResponsesOptionalConfig::default(),
            },
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: ResponsesConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a CreateResponseRequest from the conversation state.
    ///
    /// 根据对话状态构建 CreateResponseRequest。
    fn build_request(&self, state: ConversationState) -> Result<CreateResponseRequest> {
        let ConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.into_iter().map(|t| t.to_openai()).collect())
        };

        let mut input_items: Vec<InputItem> = Vec::new();

        for msg in messages {
            input_items.push(OpenAIResponseMapper::from_core_message(msg).map_err(AgentError::Mapper)?);
        }

        let input_param = InputParam::List(input_items);
        let tc = tool_choice.map(|tc| tc.to_openai());

        let mut config = self.config.clone();
        if let Some(prompt) = system_prompt {
            if config.optional.instructions.is_none() {
                config.optional.instructions = Some(prompt);
            }
        }

        Ok(config.to_request(input_param, tools, tc, None))
    }
}

impl<T: Transport> ChatAgent for ResponsesAgent<T> {
    type Stream = crate::stream::MessageStream<T::Stream, crate::agent::openai::v1::chat_completions::OpenAIProcessor>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<T::StreamFuture, crate::agent::openai::v1::chat_completions::OpenAIProcessor>
    where
        Self: 'a;

    /// Send a response request to OpenAI.
    ///
    /// 发送 Response 请求到 OpenAI。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state)?;

        let transport_req =
            TransportRequest::new(Method::Post, self.config.required.endpoint.clone(), request);
        let response: Response = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        let core_message: Message =
            OpenAIResponseMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a response request to OpenAI and receive a stream of chunks.
    ///
    /// 发送 Response 请求到 OpenAI 并接收流式响应。
    fn chat_stream<'a>(&'a self, _state: ConversationState) -> Self::ChatStreamFuture<'a> {
        todo!("Streaming for OpenAI Response API is not yet implemented")
    }
}
