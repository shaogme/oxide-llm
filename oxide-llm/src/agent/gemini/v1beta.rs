use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use oxide_llm_core::message::{ChatStream, ChatStreamWrapper, DeltaMessage, Message};
use oxide_llm_core::state::ConversationState;
use oxide_llm_core::transport::{Method, Transport, TransportError, TransportRequest};
use oxide_llm_proto::gemini::v1beta::request::{
    GenerateContentRequest, GenerationConfig, SafetySetting,
};
use oxide_llm_proto::gemini::v1beta::response::GenerateContentResponse;
use oxide_llm_proto::gemini::v1beta::{Content, Part, Tool as GeminiTool, ToolConfig};

use crate::ChatAgent;
use crate::error::{AgentError, Result};

/// Configuration for Gemini Agent (Required).
///
/// Gemini 代理配置 (必须)。
#[derive(Debug, Clone)]
pub struct GeminiRequiredConfig {
    pub model: String,
    pub endpoint: String,
}

/// Configuration for Gemini Agent (Optional).
///
/// Gemini 代理配置 (选填)。
#[derive(Debug, Clone, Default)]
pub struct GeminiOptionalConfig {
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
pub struct GeminiConfig {
    pub required: GeminiRequiredConfig,
    pub optional: GeminiOptionalConfig,
}

impl GeminiConfig {
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
/// 负责处理与 Google Gemini API 的基本交互。
#[derive(Clone)]
pub struct GeminiAgent<T: Clone> {
    /// The transport layer for network communication.
    ///
    /// 用于网络通信的传输层。
    transport: T,
    /// The configuration for the request.
    ///
    /// 请求的配置。
    config: GeminiConfig,
}

impl<T: Transport> GeminiAgent<T> {
    /// Create a new GeminiAgent.
    ///
    /// 创建一个新的 GeminiAgent。
    pub fn new(transport: T, required: GeminiRequiredConfig) -> Self {
        Self {
            transport,
            config: GeminiConfig {
                required,
                optional: GeminiOptionalConfig::default(),
            },
        }
    }

    /// Set the configuration for the agent.
    ///
    /// 设置代理的配置。
    pub fn with_config(mut self, config: GeminiConfig) -> Self {
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
            .map(|msg| msg.try_into())
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

impl<T: Transport> ChatAgent for GeminiAgent<T> {
    /// Send a chat request to Gemini.
    ///
    /// 发送聊天请求到 Gemini。
    async fn chat(&self, state: ConversationState) -> Result<Message> {
        let request = self.build_request(state)?;

        // Send Request
        let transport_req =
            TransportRequest::new(Method::Post, &self.config.required.endpoint, request);

        let response: GenerateContentResponse = self
            .transport
            .send(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        // Convert Response back to Core Message
        let core_message: Message = response.try_into().map_err(AgentError::Mapper)?;

        Ok(core_message)
    }

    /// Send a chat request to Gemini and receive a stream of chunks.
    ///
    /// 发送聊天请求到 Gemini 并接收流式响应。
    async fn chat_stream<'a>(
        &'a self,
        state: ConversationState,
    ) -> Result<ChatStreamWrapper<'a, AgentError>> {
        let request = self.build_request(state)?;

        // Send Stream Request.
        // We use the endpoint as is, assuming it's the correct stream endpoint.
        // We append "?alt=sse" to enable Server-Sent Events streaming.
        let mut endpoint = self.config.required.endpoint.clone();
        if endpoint.contains('?') {
            endpoint.push_str("&alt=sse");
        } else {
            endpoint.push_str("?alt=sse");
        }

        let transport_req = TransportRequest::new(Method::Post, &endpoint, request);

        let stream = self
            .transport
            .stream(transport_req)
            .await
            .map_err(AgentError::Transport)?;

        // Parse SSE Stream
        let parsed_stream = parse_sse_stream(stream);
        Ok(ChatStream::new(Box::pin(parsed_stream)))
    }
}

fn parse_sse_stream(
    stream: futures::stream::BoxStream<'static, std::result::Result<Bytes, TransportError>>,
) -> impl Stream<Item = Result<DeltaMessage>> {
    futures::stream::unfold(
        (stream, BytesMut::new()),
        |(mut stream, mut buffer)| async move {
            loop {
                // Check if buffer contains double newline
                if let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                    // Found an event block
                    let block = buffer.split_to(pos + 2); // Remove block from buffer

                    let s = match std::str::from_utf8(&block) {
                        Ok(s) => s,
                        Err(e) => {
                            return Some((Err(AgentError::Utf8(e)), (stream, buffer)));
                        }
                    };

                    let mut chunk_to_yield = None;
                    // Gemini doesn't strictly send [DONE], but checking doesn't hurt.
                    let mut done = false;

                    for line in s.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            let data = data.trim();
                            if data == "[DONE]" {
                                done = true;
                                break;
                            }
                            // Parse JSON
                            match serde_json::from_str::<GenerateContentResponse>(data) {
                                Ok(chunk) => {
                                    // Convert to DeltaMessage
                                    match chunk.try_into() {
                                        Ok(delta) => {
                                            chunk_to_yield = Some(delta);
                                        }
                                        Err(e) => {
                                            return Some((
                                                Err(AgentError::Mapper(e)),
                                                (stream, buffer),
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Some((Err(AgentError::Json(e)), (stream, buffer)));
                                }
                            }
                        }
                    }

                    if done {
                        return None;
                    }

                    if let Some(chunk) = chunk_to_yield {
                        return Some((Ok(chunk), (stream, buffer)));
                    } else {
                        // Keep-alive or empty block, continue loop
                        continue;
                    }
                }

                // Read more
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.extend_from_slice(&chunk);
                    }
                    Some(Err(e)) => {
                        return Some((Err(AgentError::Transport(e)), (stream, buffer)));
                    }
                    None => {
                        // EOF
                        return None;
                    }
                }
            }
        },
    )
}
