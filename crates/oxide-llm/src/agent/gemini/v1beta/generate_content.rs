use oxide_llm_core::mapper::gemini::v1beta::{
    GeminiGenerateContentMapper, GeminiGenerateContentStreamMapper,
    GenerateContentConversationState,
};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::gemini::v1beta::generate_content::request::GenerateContentRequest;
use oxide_llm_proto::gemini::v1beta::generate_content::response::GenerateContentResponse;
use oxide_llm_proto::gemini::v1beta::generate_content::{Content, Part, Tool as GeminiTool};

use crate::ChatAgent;
use crate::error::{AgentError, Result};

pub mod config;

pub use config::{
    GenerateContentConfig, GenerateContentOptionalConfig, GenerateContentRequiredConfig,
};

/// Gemini Agent.
///
/// Gemini 代理。
/// 负责处理与 Google Gemini GenerateContent API 的基本交互。
#[derive(Clone)]
pub struct GenerateContentAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The configuration for the request.
    ///
    /// 请求的配置。
    config: GenerateContentConfig,
}

pub type GenerateContentAgentBuilder<T> =
    crate::agent::builder::AgentBuilder<T, GenerateContentConfig, GenerateContentAgent<T>>;

impl<T: Transport> GenerateContentAgentBuilder<T> {
    /// Build the `GenerateContentAgent`.
    ///
    /// 构建 `GenerateContentAgent`。
    pub fn build(self) -> Result<GenerateContentAgent<T>> {
        let (transport, config) = self.build_config()?;
        Ok(GenerateContentAgent { transport, config })
    }
}

impl<T: Transport> GenerateContentAgent<T> {
    /// Create a new builder for GenerateContentAgent.
    ///
    /// 创建 GenerateContentAgent 的构建器。
    pub fn builder(transport: T) -> GenerateContentAgentBuilder<T> {
        GenerateContentAgentBuilder::new(transport)
    }

    /// Build a GenerateContentRequest from the raw conversation state.
    ///
    /// 根据底层原始对话状态构建 GenerateContentRequest。
    fn build_request(
        &self,
        state: GenerateContentConversationState,
    ) -> Result<GenerateContentRequest> {
        let GenerateContentConversationState {
            system_prompt,
            messages,
            tools,
            tool_choice,
        } = state;

        // System Prompt Conversion
        let system_instruction = system_prompt.map(|s| Content {
            parts: vec![Part::text(s.to_string())],
            role: None, // System instruction role is typically implied or None
        });

        // Tools Conversion
        let gemini_tools = if tools.is_empty() {
            None
        } else {
            Some(vec![GeminiTool {
                function_declarations: Some(tools),
                google_search_retrieval: None,
                code_execution: None,
                google_search: None,
                computer_use: None,
                url_context: None,
                file_search: None,
                mcp_servers: None,
                google_maps: None,
            }])
        };

        Ok(self
            .config
            .clone()
            .to_request(messages, system_instruction, gemini_tools, tool_choice))
    }
}

// Stream for Gemini Messages.
//
// Gemini Messages 流。

/// SSE Processor for Gemini GenerateContent raw stream events.
///
/// Gemini GenerateContent 裸事件的 SSE 处理器。
pub struct RawGeminiProcessor;

impl RawGeminiProcessor {
    /// Creates a new `RawGeminiProcessor`.
    ///
    /// 创建一个新的 `RawGeminiProcessor`。
    pub fn new() -> Self {
        Self
    }
}

impl Default for RawGeminiProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor<GenerateContentResponse> for RawGeminiProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<GenerateContentResponse>>, bool) {
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
                match serde_json::from_str::<GenerateContentResponse>(data) {
                    Ok(chunk) => {
                        chunk_to_yield = Some(Ok(chunk));
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

impl crate::stream::StreamMapper<GenerateContentResponse> for GeminiGenerateContentStreamMapper {
    fn map_item(&mut self, raw: GenerateContentResponse) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for GenerateContentAgent<T> {
    type RawConversationState = GenerateContentConversationState;
    type RawMessage = GenerateContentResponse;
    type RawDelta = GenerateContentResponse;
    type RawStream =
        crate::stream::MessageStream<T::Stream, RawGeminiProcessor, GenerateContentResponse>;
    type ChatStreamRawFuture<'a>
        = crate::stream::AgentChatStreamRawFuture<
        T::StreamFuture,
        RawGeminiProcessor,
        GenerateContentResponse,
    >
    where
        Self: 'a;

    type Stream = crate::stream::MappedStream<
        Self::RawStream,
        GeminiGenerateContentStreamMapper,
        Self::RawDelta,
    >;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
        Self::ChatStreamRawFuture<'a>,
        GeminiGenerateContentStreamMapper,
        Self::RawDelta,
    >
    where
        Self: 'a;

    /// Send a chat request to Gemini and return raw response.
    ///
    /// 发送聊天请求到 Gemini 并返回原始响应。
    async fn chat_raw(&self, state: Self::RawConversationState) -> Result<GenerateContentResponse> {
        let request = self.build_request(state)?;

        let endpoint = format!("{}:generateContent", self.config.required().endpoint());
        let transport_req = TransportRequest::new(Method::Post, endpoint, request);
        let response: GenerateContentResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        Ok(response)
    }

    /// Send a chat request to Gemini and receive a stream of raw chunks with configuration.
    ///
    /// 发送聊天请求到 Gemini 并接收带有配置的原始块的流式响应。
    fn chat_stream_raw_with<'a>(
        &'a self,
        state: Self::RawConversationState,
        mut config: crate::ChatStreamRawConfig<Self::RawDelta>,
    ) -> Self::ChatStreamRawFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let request_res = self.build_request(state);
        let fut = request_res.map(|request| {
            let endpoint = format!(
                "{}:streamGenerateContent?alt=sse",
                self.config.required().endpoint()
            );
            let transport_req = TransportRequest::new(Method::Post, endpoint.clone(), request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::with_hook(
            fut,
            RawGeminiProcessor::new(),
            on_raw_delta,
        )
    }

    /// Send a chat request to Gemini and receive a stream of raw chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收原始块的流式响应。
    fn chat_stream_raw<'a>(
        &'a self,
        state: Self::RawConversationState,
    ) -> Self::ChatStreamRawFuture<'a> {
        self.chat_stream_raw_with(state, Default::default())
    }

    /// Send a chat request to Gemini.
    ///
    /// 发送聊天请求到 Gemini。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let raw_state =
            GenerateContentConversationState::try_from(state).map_err(AgentError::Mapper)?;
        let response = self.chat_raw(raw_state).await?;

        // Convert Response back to Core Message
        let core_message: Message =
            GeminiGenerateContentMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Gemini and receive a stream of chunks with configuration.
    ///
    /// 发送聊天请求到 Gemini 并接收带有配置的流式响应。
    fn chat_stream_with<'a>(
        &'a self,
        state: ConversationState,
        mut config: crate::ChatStreamConfig<Self::RawDelta>,
    ) -> Self::ChatStreamFuture<'a> {
        let on_raw_delta = config.take_on_raw_delta();
        let on_delta = config.take_on_delta();
        let raw_state_res =
            GenerateContentConversationState::try_from(state).map_err(AgentError::Mapper);
        let raw_stream_fut = match raw_state_res {
            Ok(raw_state) => self.chat_stream_raw(raw_state),
            Err(e) => crate::stream::AgentChatStreamRawFuture::with_hook(
                Err(e),
                RawGeminiProcessor::new(),
                None,
            ),
        };
        crate::stream::AgentChatStreamFuture::with_hooks(
            raw_stream_fut,
            GeminiGenerateContentStreamMapper::new(),
            on_raw_delta,
            on_delta,
        )
    }

    /// Send a chat request to Gemini and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        self.chat_stream_with(state, crate::ChatStreamConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::SseProcessor;

    #[test]
    fn test_raw_gemini_processor() {
        let mut processor = RawGeminiProcessor::new();
        let sse_data = r#"data: {"candidates":[{"content":{"parts":[{"text":"Hello raw"}]}}]}"#;

        let (res, done) = processor.process(sse_data.as_bytes());
        assert!(!done);
        assert!(res.is_some());
        let raw_resp = res.unwrap().unwrap();
        assert_eq!(
            raw_resp.candidates[0].content.parts[0].text,
            Some("Hello raw".to_string())
        );
    }
}
