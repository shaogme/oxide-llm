use oxide_llm_core::mapper::openai::v1::{OpenAIResponseMapper, OpenAIResponseStreamMapper};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::openai::v1::response::chunk::ResponseStreamEvent;
use oxide_llm_proto::openai::v1::response::request::{
    CreateResponseRequest, InputItem, InputParam,
};
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
    fn build_request(
        &self,
        state: ConversationState,
        stream: bool,
    ) -> Result<CreateResponseRequest> {
        let ConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        let tools = if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .iter()
                    .map(OpenAIResponseMapper::tool_to_openai_response)
                    .collect(),
            )
        };

        let mut input_items: Vec<InputItem> = Vec::new();

        for msg in messages {
            input_items
                .extend(OpenAIResponseMapper::from_core_message(msg).map_err(AgentError::Mapper)?);
        }

        let input_param = InputParam::List(input_items);
        let tc = tool_choice.as_ref().map(OpenAIResponseMapper::tool_choice_to_openai_response);

        let mut config = self.config.clone();
        if let Some(prompt) = system_prompt
            && config.optional().instructions().is_none()
        {
            config.optional_mut().set_instructions(Some(prompt));
        }

        let mut request = config.to_request(input_param, tools, tc, None);
        if stream {
            request.stream = Some(true);
        }

        Ok(request)
    }
}

/// SSE Processor for OpenAI Response raw stream events.
///
/// OpenAI Response 裸事件的 SSE 处理器。
pub struct RawOpenAIResponseProcessor;

impl RawOpenAIResponseProcessor {
    /// Creates a new `RawOpenAIResponseProcessor`.
    ///
    /// 创建一个新的 `RawOpenAIResponseProcessor`。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawOpenAIResponseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor<ResponseStreamEvent> for RawOpenAIResponseProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<ResponseStreamEvent>>, bool) {
        let s = match std::str::from_utf8(block) {
            Ok(s) => s,
            Err(e) => return (Some(Err(AgentError::Utf8(e))), false),
        };

        let mut chunk_to_yield = None;
        let mut done = false;

        for line in s.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();
                if data == "[DONE]" {
                    done = true;
                    break;
                }
                match serde_json::from_str::<ResponseStreamEvent>(data) {
                    Ok(event) => {
                        chunk_to_yield = Some(Ok(event));
                    }
                    Err(e) => return (Some(Err(AgentError::Json(e))), false),
                }
            }
        }

        if done {
            return (chunk_to_yield, true);
        }

        (chunk_to_yield, false)
    }
}

impl crate::stream::StreamMapper<ResponseStreamEvent> for OpenAIResponseStreamMapper {
    fn map_item(&mut self, raw: ResponseStreamEvent) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for ResponsesAgent<T> {
    type RawMessage = Response;
    type RawDelta = ResponseStreamEvent;
    type RawStream =
        crate::stream::MessageStream<T::Stream, RawOpenAIResponseProcessor, ResponseStreamEvent>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<
            T::StreamFuture,
            RawOpenAIResponseProcessor,
            ResponseStreamEvent,
        >
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<Self::RawStream, OpenAIResponseStreamMapper>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
            Self::ChatStreamRawFuture<'a>,
            OpenAIResponseStreamMapper,
        >
    where
        Self: 'a;

    /// Send a response request to OpenAI and return raw response.
    ///
    /// 发送 Response 请求到 OpenAI 并返回原始响应。
    async fn chat_raw(&self, state: ConversationState) -> Result<Response> {
        let request = self.build_request(state, false)?;

        let transport_req = TransportRequest::new(
            Method::Post,
            self.config.required().endpoint().to_string(),
            request,
        );
        let response: Response = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        Ok(response)
    }

    /// Send a response request to OpenAI and receive a stream of raw chunks.
    ///
    /// 发送 Response 请求到 OpenAI 并接收原始块的流式响应。
    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        let request_res = self.build_request(state, true);
        let fut = request_res.map(|request| {
            let transport_req = TransportRequest::new(
                Method::Post,
                self.config.required().endpoint().to_string(),
                request,
            );
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::new(fut, RawOpenAIResponseProcessor::new())
    }

    /// Send a response request to OpenAI.
    ///
    /// 发送 Response 请求到 OpenAI。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let response = self.chat_raw(state).await?;

        let core_message: Message =
            OpenAIResponseMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a response request to OpenAI and receive a stream of chunks.
    ///
    /// 发送 Response 请求到 OpenAI 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        crate::stream::AgentChatStreamFuture::new(
            self.chat_stream_raw(state),
            OpenAIResponseStreamMapper::new(),
        )
    }
}
