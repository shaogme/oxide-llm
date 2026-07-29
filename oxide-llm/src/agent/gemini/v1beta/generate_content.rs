use oxide_llm_core::mapper::gemini::v1beta::{GeminiMapper, GeminiStreamMapper};
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
                .map(GeminiMapper::tool_to_gemini_function_declaration)
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
            .map(GeminiMapper::from_core_message)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(AgentError::Mapper)?;

        // Tool Choice Conversion
        let tool_config_override = tool_choice
            .as_ref()
            .and_then(GeminiMapper::tool_choice_to_gemini);

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

/// SSE Processor for Gemini GenerateContent stream events.
///
/// Gemini GenerateContent 流事件的 SSE 处理器。
pub struct GeminiProcessor {
    mapper: GeminiStreamMapper,
}

impl GeminiProcessor {
    /// Creates a new `GeminiProcessor`.
    ///
    /// 创建一个新的 `GeminiProcessor`。
    pub fn new() -> Self {
        Self {
            mapper: GeminiStreamMapper::new(),
        }
    }
}

impl Default for GeminiProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::stream::SseProcessor for GeminiProcessor {
    fn process(&mut self, block: &[u8]) -> (Option<Result<DeltaMessage>>, bool) {
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
                    Ok(chunk) => match self.mapper.map_response(chunk) {
                        Ok(delta) => {
                            chunk_to_yield = Some(Ok(delta));
                        }
                        Err(e) => return (Some(Err(AgentError::Mapper(e))), false),
                    },
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

impl<T: Transport> ChatAgent for GenerateContentAgent<T> {
    type Stream = crate::stream::MessageStream<T::Stream, GeminiProcessor>;
    type ChatStreamFuture<'a>
        = crate::stream::AgentChatStreamFuture<T::StreamFuture, GeminiProcessor>
    where
        Self: 'a;

    /// Send a chat request to Gemini.
    ///
    /// 发送聊天请求到 Gemini。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state)?;

        // Send Request
        let endpoint = format!("{}:generateContent", self.config.required().endpoint());
        let transport_req = TransportRequest::new(Method::Post, endpoint.clone(), request);
        let response: GenerateContentResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message =
            GeminiMapper::to_core_message(response).map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Gemini and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收流式响应。
    fn chat_stream<'a>(&'a self, state: ConversationState) -> Self::ChatStreamFuture<'a> {
        let request_res = self.build_request(state);
        let fut = request_res.map(|request| {
            let endpoint = format!(
                "{}:streamGenerateContent?alt=sse",
                self.config.required().endpoint()
            );
            let transport_req = TransportRequest::new(Method::Post, endpoint.clone(), request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamFuture::new(fut, GeminiProcessor::new())
    }
}
