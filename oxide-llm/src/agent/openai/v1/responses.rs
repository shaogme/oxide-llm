use oxide_llm_core::mapper::openai::v1::OpenAIResponseMapper;
use oxide_llm_core::message::Message;
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::tool::{ToolAdapter, ToolChoiceAdapter};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::response::request::{CreateResponseRequest, InputItem, InputParam};
use oxide_llm_proto::openai::v1::response::response::Response;

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::{ResponsesConfig, ResponsesOptionalConfig, ResponsesRequiredConfig};

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
            config: ResponsesConfig::new(required),
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
            if config.optional().instructions().is_none() {
                config.optional_mut().set_instructions(Some(prompt));
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
            TransportRequest::new(Method::Post, self.config.required().endpoint().to_string(), request);
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
