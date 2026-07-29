use oxide_llm_core::{
    mapper::gemini::v1beta::{GeminiInteractionsMapper, GeminiInteractionsStreamMapper},
    message::{DeltaMessage, Message},
    state::ConversationState,
    transport::{Method, Transport, TransportRequest},
};
use oxide_llm_proto::gemini::v1beta::interactions::{
    request::CreateInteractionRequest, sse::InteractionSseEvent,
};

use crate::{
    ChatAgent,
    error::{AgentError, Result},
    stream::SseProcessor,
};

pub mod config;

pub use config::{
    InteractionsConfig, InteractionsOptionalConfig, InteractionsRequiredConfig,
};

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

impl<T: Transport> InteractionsAgent<T> {
    /// Create a new InteractionsAgent.
    ///
    /// 创建一个新的 InteractionsAgent。
    pub fn new(transport: T, required: InteractionsRequiredConfig) -> Self {
        Self {
            transport,
            config: InteractionsConfig::new(required),
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: InteractionsConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a CreateInteractionRequest from the conversation state.
    ///
    /// 根据对话状态构建 CreateInteractionRequest。
    fn build_request(
        &self,
        state: ConversationState,
        stream_override: Option<bool>,
    ) -> Result<CreateInteractionRequest> {
        let sys_prompt = state.system_prompt;
        let messages = state.messages;
        let tools = if state.tools.is_empty() {
            None
        } else {
            Some(state.tools)
        };
        let tool_choice = state.tool_choice;

        let base_req = GeminiInteractionsMapper::from_core_messages(
            messages,
            self.config.required().model_static(),
            tools,
            tool_choice,
        )
        .map_err(AgentError::Mapper)?;

        let final_req = self.config.apply_to_request(
            base_req,
            stream_override,
            sys_prompt.as_deref(),
        );

        Ok(final_req)
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
                match serde_json::from_str::<InteractionSseEvent>(data) {
                    Ok(event) => {
                        if matches!(
                            &event,
                            InteractionSseEvent::InteractionCompleted(_)
                                | InteractionSseEvent::Error(_)
                        ) {
                            done = true;
                        }
                        chunk_to_yield = Some(Ok(event));
                    }
                    Err(e) => return (Some(Err(AgentError::Json(e))), false),
                }
            }
        }

        (chunk_to_yield, done)
    }
}

impl crate::stream::StreamMapper<InteractionSseEvent> for GeminiInteractionsStreamMapper {
    fn map_item(&mut self, raw: InteractionSseEvent) -> Result<Option<DeltaMessage>> {
        self.map_event(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for InteractionsAgent<T> {
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

    type Stream = crate::stream::MappedStream<Self::RawStream, GeminiInteractionsStreamMapper>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
            Self::ChatStreamRawFuture<'a>,
            GeminiInteractionsStreamMapper,
        >
    where
        Self: 'a;

    /// Send a chat request to Gemini Interactions API and return raw SSE events.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并返回原始 SSE 事件列表。
    async fn chat_raw(&self, state: ConversationState) -> Result<Vec<InteractionSseEvent>> {
        let request = self.build_request(state, None)?;

        let endpoint = self.config.required().endpoint().to_string();
        let transport_req = TransportRequest::new(Method::Post, endpoint, request);

        use futures::StreamExt;

        let mut stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        let mut processor = RawInteractionsProcessor::new();
        let mut events = Vec::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk_bytes = chunk_res.map_err(AgentError::Transport)?;
            let (event_opt, done) = processor.process(&chunk_bytes);
            if let Some(event_res) = event_opt {
                let event = event_res?;
                events.push(event);
            }
            if done {
                break;
            }
        }

        Ok(events)
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of raw SSE events.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收原始 SSE 事件的流。
    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        let request_res = self.build_request(state, Some(true));
        let fut = request_res.map(|request| {
            let endpoint = self.config.required().endpoint().to_string();
            let transport_req = TransportRequest::new(Method::Post, endpoint, request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::new(fut, RawInteractionsProcessor::new())
    }

    /// Send a chat request to Gemini Interactions API.
    ///
    /// 发送聊天请求到 Gemini Interactions API。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let events = self.chat_raw(state).await?;
        let mut mapper = GeminiInteractionsStreamMapper::new();
        let mut assembler = oxide_llm_core::message::MessageAssembler::new();

        for event in events {
            let delta = mapper.map_event(event).map_err(AgentError::Mapper)?;
            assembler.add(delta);
        }

        Ok(assembler.build())
    }

    /// Send a chat request to Gemini Interactions API and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini Interactions API 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        crate::stream::AgentChatStreamFuture::new(
            self.chat_stream_raw(state),
            GeminiInteractionsStreamMapper::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_core::message::{ContentPart, DeltaContentPart, Role};
    use ref_str::StaticRefStr;

    #[test]
    fn test_interactions_config_build_request() {
        let required = InteractionsRequiredConfig::new(
            "https://generativelanguage.googleapis.com/v1beta/interactions",
        )
        .with_model("gemini-3.6-flash");

        let config = InteractionsConfig::new(required);

        let mut state = ConversationState::new(Some(StaticRefStr::from("System prompt test")));
        state.add_message(Message {
            role: Role::User,
            content: vec![ContentPart::Text {
                text: "Hello".into(),
                signature: None,
            }],
            name: None,
        });

        let messages = state.messages.clone();
        let sys_prompt = state.system_prompt.clone();
        let base_req = GeminiInteractionsMapper::from_core_messages(
            messages,
            config.required().model_static(),
            None,
            None,
        )
        .unwrap();

        let req = config.apply_to_request(base_req, Some(true), sys_prompt.as_deref());

        assert_eq!(req.model.as_deref(), Some("gemini-3.6-flash"));
        assert_eq!(req.system_instruction.as_deref(), Some("System prompt test"));
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
