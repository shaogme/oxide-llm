use oxide_llm_core::mapper::gemini::v1beta::{GeminiMapper, GeminiStreamMapper};
use oxide_llm_core::message::{DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::tool::{ToolAdapter, ToolChoiceAdapter};
use oxide_llm_core::transport::{Method, Transport, TransportRequest};
use oxide_llm_proto::gemini::v1beta::generate_content::request::{
    GenerateContentRequest, GenerationConfig, SafetySetting,
};
use oxide_llm_proto::gemini::v1beta::generate_content::response::GenerateContentResponse;
use oxide_llm_proto::gemini::v1beta::generate_content::{
    Content, Part, Tool as GeminiTool, ToolConfig,
};

use crate::ChatAgent;
use crate::error::{AgentError, Result};

/// Configuration for Gemini Agent (Required).
///
/// Gemini 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct GenerateContentRequiredConfig {
    pub model: String,
    pub endpoint: String,
}

/// Configuration for Gemini Agent (Optional).
///
/// Gemini 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct GenerateContentOptionalConfig {
    pub safety_settings: Option<Vec<SafetySetting>>,
    pub system_instruction: Option<Content>,
    pub tool_config: Option<ToolConfig>,
    pub cached_content: Option<String>,

    // Generation Config fields
    pub stop_sequences: Option<Vec<String>>,
    pub response_mime_type: Option<String>,
    pub max_output_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub response_logprobs: Option<bool>,
    pub logprobs: Option<i32>,
}

/// Configuration for Gemini Agent.
///
/// Gemini 代理配置。
#[derive(Debug, Clone)]
pub struct GenerateContentConfig {
    pub required: GenerateContentRequiredConfig,
    pub optional: GenerateContentOptionalConfig,
}

impl GenerateContentConfig {
    /// Convert Config to GenerateContentRequest with provided contents.
    ///
    /// 将配置转换为 GenerateContentRequest，并填入内容。
    pub fn to_request(
        self,
        contents: Vec<Content>,
        system_instruction_override: Option<Content>,
        tools: Option<Vec<GeminiTool>>,
        tool_config_override: Option<ToolConfig>,
    ) -> GenerateContentRequest {
        let generation_config = Some(GenerationConfig {
            stop_sequences: self.optional.stop_sequences,
            response_mime_type: self.optional.response_mime_type,
            response_schema: None, // Can be added to optional config if needed
            candidate_count: None,
            max_output_tokens: self.optional.max_output_tokens,
            temperature: self.optional.temperature,
            top_p: self.optional.top_p,
            top_k: self.optional.top_k,
            seed: None,
            presence_penalty: self.optional.presence_penalty,
            frequency_penalty: self.optional.frequency_penalty,
            response_logprobs: self.optional.response_logprobs,
            logprobs: self.optional.logprobs,
            speech_config: None,
            thinking_config: None,
            image_config: None,
            media_resolution: None,
            response_json_schema: None,
            response_modalities: None,
        });

        GenerateContentRequest {
            contents,
            tools,
            tool_config: tool_config_override.or(self.optional.tool_config),
            safety_settings: self.optional.safety_settings,
            system_instruction: system_instruction_override.or(self.optional.system_instruction),
            generation_config,
            cached_content: self.optional.cached_content,
        }
    }
}

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
            config: GenerateContentConfig {
                required,
                optional: GenerateContentOptionalConfig::default(),
            },
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
            parts: vec![Part::text(s)],
            role: None, // System instruction role is typically implied or None
        });

        // Tools Conversion
        let gemini_tools = if tools.is_empty() {
            None
        } else {
            let function_declarations: Vec<_> = tools
                .iter()
                .map(|t| t.to_gemini_function_declaration())
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
        let tool_config_override = tool_choice.and_then(|tc| tc.to_gemini());

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
    type ChatStreamFuture<'a> = crate::stream::AgentChatStreamFuture<T::StreamFuture, GeminiProcessor> where Self: 'a;

    /// Send a chat request to Gemini.
    ///
    /// 发送聊天请求到 Gemini。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state)?;

        // Send Request
        let endpoint = format!("{}:generateContent", self.config.required.endpoint);
        let transport_req = TransportRequest::new(Method::Post, &endpoint, request);
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
    fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> Self::ChatStreamFuture<'a> {
        let request_res = self.build_request(state);
        let fut = request_res.map(|request| {
            let endpoint = format!("{}:streamGenerateContent?alt=sse", self.config.required.endpoint);
            let transport_req = TransportRequest::new(Method::Post, &endpoint, request);
            self.transport.stream(transport_req)
        });
        crate::stream::AgentChatStreamFuture::new(fut, GeminiProcessor::new())
    }
}
