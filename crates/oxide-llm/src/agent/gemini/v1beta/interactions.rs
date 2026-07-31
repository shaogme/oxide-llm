use oxide_llm_core::{
    mapper::gemini::v1beta::{GeminiInteractionsStreamMapper, InteractionsConversationState},
    message::{DeltaMessage, Message},
    state::ConversationState,
    transport::{Method, Transport, TransportRequest},
};
use oxide_llm_proto::gemini::v1beta::interactions::{
    request::{CreateInteractionRequest, InteractionsInput},
    sse::InteractionSseEvent,
};

use crate::{
    ChatAgent,
    error::{AgentError, Result},
};

pub mod config;

pub use config::InteractionsConfig;

/// Gemini Interactions Agent.
///
/// Gemini Interactions 代理。
/// 负责处理与 Google Gemini Interactions API 的交互。
#[derive(Clone)]
pub struct InteractionsAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The configuration for the request.
    ///
    /// 请求的配置。
    config: InteractionsConfig,
}

pub type InteractionsAgentBuilder<T> =
    crate::agent::builder::AgentBuilder<T, InteractionsConfig, InteractionsAgent<T>>;

impl<T: Transport> InteractionsAgentBuilder<T> {
    /// Build the `InteractionsAgent`.
    ///
    /// 构建 `InteractionsAgent`。
    pub fn build(self) -> Result<InteractionsAgent<T>> {
        let (transport, config) = self.build_config()?;
        Ok(InteractionsAgent { transport, config })
    }
}

impl<T: Transport> InteractionsAgent<T> {
    /// Create a new builder for InteractionsAgent.
    ///
    /// 创建 InteractionsAgent 的构建器。
    pub fn builder(transport: T) -> InteractionsAgentBuilder<T> {
        InteractionsAgentBuilder::new(transport)
    }

    /// Build a CreateInteractionRequest from the raw conversation state.
    ///
    /// 根据底层原始对话状态构建 CreateInteractionRequest。
    fn build_request(
        &self,
        state: InteractionsConversationState,
        stream_override: Option<bool>,
    ) -> Result<CreateInteractionRequest> {
        let InteractionsConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        let input = InteractionsInput::Steps(messages);
        let mapped_tools = if tools.is_empty() { None } else { Some(tools) };

        let request = self.config.to_request(
            input,
            mapped_tools,
            tool_choice,
            stream_override,
            system_prompt.as_deref(),
        );

        Ok(request)
    }
}

/// SSE Processor for Gemini Interactions raw stream events.
///
/// Gemini Interactions 裸事件的 SSE 处理器。
pub struct RawInteractionsProcessor;

impl RawInteractionsProcessor {
    /// Creates a new `RawInteractionsProcessor`.
    ///
    /// 创建一个新的 `RawInteractionsProcessor`。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawInteractionsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor<InteractionSseEvent> for RawInteractionsProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<InteractionSseEvent>>, bool) {
        crate::stream::parse_json_sse_block(block, |event| {
            matches!(
                event,
                InteractionSseEvent::InteractionCompleted(_)
                    | InteractionSseEvent::Error(_)
            )
        })
    }
}

impl crate::stream::StreamMapper<InteractionSseEvent> for GeminiInteractionsStreamMapper {
    fn map_item(&mut self, raw: InteractionSseEvent) -> Result<Option<DeltaMessage>> {
        self.map_event(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for InteractionsAgent<T> {
    type RawConversationState = InteractionsConversationState;
    type RawMessage = Vec<InteractionSseEvent>;
    type RawDelta = InteractionSseEvent;
    type RawStream =
        crate::stream::MessageStream<T::Stream, RawInteractionsProcessor, InteractionSseEvent>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<
        T::StreamFuture,
        RawInteractionsProcessor,
        InteractionSseEvent,
    >
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<
        Self::RawStream,
        GeminiInteractionsStreamMapper,
        Self::RawDelta,
    >;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
        Self::ChatStreamRawFuture<'a>,
        GeminiInteractionsStreamMapper,
        Self::RawDelta,
    >
    where
        Self: 'a;

    /// Send a chat request to Gemini Interactions API and return raw SSE events.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并返回原始 SSE 事件。
    async fn chat_raw(
        &self,
        state: Self::RawConversationState,
    ) -> Result<Vec<InteractionSseEvent>> {
        let request = self.build_request(state, Some(true))?;

        let endpoint = self.config.endpoint().unwrap_or("interactions").to_string();
        let transport_req = TransportRequest::new(Method::Post, endpoint, request);
        let stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        use futures::StreamExt;
        let mut message_stream =
            crate::stream::MessageStream::new(stream, RawInteractionsProcessor::new());

        let mut events = Vec::new();
        while let Some(item) = message_stream.next().await {
            let event = item?;
            events.push(event);
        }

        Ok(events)
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of raw SSE events with configuration.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收带有配置的原始 SSE 事件的流。
    fn chat_stream_raw_with<'a>(
        &'a self,
        state: Self::RawConversationState,
        mut config: crate::ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let request_res = self.build_request(state, Some(true));
        let fut = request_res.map(|request| {
            let endpoint = self.config.endpoint().unwrap_or("interactions").to_string();
            let transport_req = TransportRequest::new(Method::Post, endpoint, request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::with_hook(
            fut,
            RawInteractionsProcessor::new(),
            on_raw_delta,
        )
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of raw SSE events.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收原始 SSE 事件的流。
    fn chat_stream_raw<'a>(
        &'a self,
        state: Self::RawConversationState,
    ) -> Self::ChatStreamRawFuture<'a> {
        self.chat_stream_raw_with(state, crate::ChatStreamRawConfig::default())
    }

    /// Send a chat request to Gemini Interactions API.
    ///
    /// 发送聊天请求到 Gemini Interactions API。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let raw_state =
            InteractionsConversationState::try_from(state).map_err(AgentError::Mapper)?;
        let events = self.chat_raw(raw_state).await?;
        let mut mapper = GeminiInteractionsStreamMapper::new();
        let mut assembler = oxide_llm_core::message::MessageAssembler::new();

        for event in events {
            let delta = mapper.map_event(event).map_err(AgentError::Mapper)?;
            assembler.add(delta);
        }

        Ok(assembler.build())
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of chunks with configuration.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收带有配置的流式响应。
    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        mut config: crate::ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let on_delta = config.take_on_delta();
        let raw_state_res =
            InteractionsConversationState::try_from(state).map_err(AgentError::Mapper);
        let raw_stream_fut = match raw_state_res {
            Ok(raw_state) => self.chat_stream_raw(raw_state),
            Err(e) => crate::stream::AgentChatStreamRawFuture::with_hook(
                Err(e),
                RawInteractionsProcessor::new(),
                None,
            ),
        };
        crate::stream::AgentChatStreamFuture::with_hooks(
            raw_stream_fut,
            GeminiInteractionsStreamMapper::new(),
            on_raw_delta,
            on_delta,
        )
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        self.chat_stream_with(state, crate::ChatStreamConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_core::message::{ContentPart, DeltaContentPart, Role};

    #[test]
    fn test_interactions_config_build_request() {
        let mut config = InteractionsConfig::new();
        config.set_model(Some("gemini-3.6-flash"));

        let mut state = ConversationState::new();
        state.set_system_prompt("System prompt test");
        state.add_message(Message {
            role: Role::User,
            content: vec![ContentPart::Text {
                text: "Hello".into(),
                signature: None,
            }],
            name: None,
        });

        let input = InteractionsInput::String("Hello".into());
        let sys_prompt = Some("System prompt test");

        let req = config.to_request(input, None, None, Some(true), sys_prompt);

        assert_eq!(req.model.as_deref(), Some("gemini-3.6-flash"));
        assert_eq!(
            req.system_instruction.as_deref(),
            Some("System prompt test")
        );
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn test_interactions_processor() {
        use crate::stream::{SseProcessor, StreamMapper};

        let mut processor = RawInteractionsProcessor::new();
        let mut mapper = GeminiInteractionsStreamMapper::new();
        let sse_data = r#"data: {"event_type":"step.delta","index":0,"delta":{"type":"text","text":"Hello world"}}"#;

        let (res, done) = processor.process(sse_data.as_bytes());
        assert!(!done);
        assert!(res.is_some());
        let raw_event = res.unwrap().unwrap();
        let delta = mapper.map_item(raw_event).unwrap().unwrap();
        assert_eq!(delta.role, Some(Role::Assistant));
        if let Some(parts) = delta.content {
            if let DeltaContentPart::Text { text, .. } = &parts[0] {
                assert_eq!(text.as_str(), "Hello world");
            } else {
                panic!("Expected Text delta");
            }
        } else {
            panic!("Expected delta content");
        }
    }
}
