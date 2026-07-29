use oxide_llm_core::mapper::gemini::v1beta::{
    GeminiGenerateContentMapper, GeminiGenerateContentStreamMapper,
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

impl<T: Transport> GenerateContentAgent<T> {
    /// Create a new GenerateContentAgent.
    ///
    /// 创建一个新的 GenerateContentAgent。
    pub fn new(transport: T, required: GenerateContentRequiredConfig) -> Self {
        Self {
            transport,
            config: GenerateContentConfig::new(required),
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: GenerateContentConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a GenerateContentRequest from the conversation state.
    ///
    /// 根据对话状态构建 GenerateContentRequest。
    fn build_request(&self, state: ConversationState) -> Result<GenerateContentRequest> {
        // state is mut if we need to consume it
        let sys_prompt = state.system_prompt;
        let messages = state.messages;
        let tools = state.tools;
        let tool_choice = state.tool_choice;

        // System Prompt Conversion
        let system_instruction = sys_prompt.map(|s| Content {
            parts: vec![Part::text(s.to_string())],
            role: None, // System instruction role is typically implied or None
        });

        // Tools Conversion
        let gemini_tools = if tools.is_empty() {
            None
        } else {
            let function_declarations: Vec<_> = tools
                .iter()
                .map(GeminiGenerateContentMapper::tool_to_gemini_function_declaration)
                .collect();

            Some(vec![GeminiTool {
                function_declarations: Some(function_declarations),
                google_search_retrieval: None,
                code_execution: None,
                google_search: None,
                computer_use: None,
                url_context: None,
                file_search: None,
                google_maps: None,
            }])
        };

        // Messages Conversion
        let contents: Vec<Content> = messages
            .into_iter()
            .map(GeminiGenerateContentMapper::from_core_message)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AgentError::Mapper)?;

        // Tool Choice Conversion
        let tool_config_override = tool_choice
            .as_ref()
            .and_then(GeminiGenerateContentMapper::tool_choice_to_gemini);

        Ok(self.config.clone().to_request(
            contents,
            system_instruction,
            gemini_tools,
            tool_config_override,
        ))
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

impl crate::stream::StreamMapper<GenerateContentResponse>
    for GeminiGenerateContentStreamMapper
{
    fn map_item(&mut self, raw: GenerateContentResponse) -> Result<Option<DeltaMessage>> {
        self.map_response(raw).map(Some).map_err(AgentError::Mapper)
    }
}

impl<T: Transport> ChatAgent for GenerateContentAgent<T> {
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

    type Stream = crate::stream::MappedStream<Self::RawStream, GeminiGenerateContentStreamMapper>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<
            Self::ChatStreamRawFuture<'a>,
            GeminiGenerateContentStreamMapper,
        >
    where
        Self: 'a;

    /// Send a chat request to Gemini and return the raw response.
    ///
    /// 发送聊天请求到 Gemini 并返回原始响应。
    async fn chat_raw(&self, state: ConversationState) -> Result<GenerateContentResponse> {
        let request = self.build_request(state)?;

        // Send Request
        let endpoint = format!("{}:generateContent", self.config.required().endpoint());
        let transport_req = TransportRequest::new(Method::Post, endpoint.clone(), request);
        let response: GenerateContentResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        Ok(response)
    }

    /// Send a chat request to Gemini and receive a stream of raw chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收原始块的流式响应。
    fn chat_stream_raw<'a>(&'a self, state: ConversationState) -> Self::ChatStreamRawFuture<'a> {
        let request_res = self.build_request(state);
        let fut = request_res.map(|request| {
            let endpoint = format!(
                "{}:streamGenerateContent?alt=sse",
                self.config.required().endpoint()
            );
            let transport_req = TransportRequest::new(Method::Post, endpoint.clone(), request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamRawFuture::new(fut, RawGeminiProcessor::new())
    }

    /// Send a chat request to Gemini.
    ///
    /// 发送聊天请求到 Gemini。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let response = self.chat_raw(state).await?;

        // Convert Response back to Core Message
        let core_message: Message =
            GeminiGenerateContentMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Gemini and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        crate::stream::AgentChatStreamFuture::new(
            self.chat_stream_raw(state),
            GeminiGenerateContentStreamMapper::new(),
        )
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
