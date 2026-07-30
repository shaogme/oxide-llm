use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};

use crate::mapper::MapperError;
use crate::message::{
    Audio, ContentPart, DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, Image,
    ImageSource, Message, Role,
};
use crate::state::{ConversationState, ConversationStateTrait};
use crate::tool::{FunctionDefinition, ToolCall, ToolChoice, ToolDefinition, ToolType};
use oxide_llm_proto::openai::v1::chat_completions::{
    FunctionDefinition as OpenAIFunctionDefinition, Tool as OpenAIChatCompletionsTool,
    ToolCall as OpenAIToolCall, ToolCallFunction, ToolChoice as OpenAIChatCompletionsToolChoice,
    ToolChoiceFunction as OpenAIChatCompletionsToolChoiceFunction,
    ToolChoiceNamed as OpenAIChatCompletionsToolChoiceNamed,
    chunk::ChatCompletionChunk as OpenAIStreamChunk,
    request::{
        ChatCompletionMessage, ContentPart as OpenAIContentPart, ImageUrl, InputAudio, UserContent,
    },
    response::ChatCompletionResponse,
};

/// Mapper for OpenAI Chat Completions protocol.
///
/// OpenAI Chat Completions 协议映射器。
pub struct OpenAIChatCompletionMapper;

impl OpenAIChatCompletionMapper {
    /// Convert core Message to OpenAI ChatCompletionMessage.
    ///
    /// 将核心 Message 转换为 OpenAI ChatCompletionMessage。
    pub fn from_core_message(msg: Message) -> Result<Vec<ChatCompletionMessage>, MapperError> {
        match msg.role {
            Role::User => {
                if msg.content.len() == 1
                    && let ContentPart::Text { text, signature: _ } = &msg.content[0]
                {
                    return Ok(vec![ChatCompletionMessage::User {
                        content: UserContent::Text(text.clone()),
                        name: msg.name,
                    }]);
                }

                let mut parts = Vec::new();
                for part in msg.content {
                    match part {
                        ContentPart::Text { text, signature: _ } => {
                            parts.push(OpenAIContentPart::Text { text });
                        }
                        ContentPart::Image(image) => {
                            parts.push(OpenAIContentPart::ImageUrl {
                                image_url: Self::convert_image_openai(image)?,
                            });
                        }
                        ContentPart::Audio(audio) => {
                            parts.push(OpenAIContentPart::InputAudio {
                                input_audio: Self::convert_audio_openai(audio)?,
                            });
                        }
                        ContentPart::Json(value) => {
                            let text =
                                serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                            parts.push(OpenAIContentPart::Text { text });
                        }
                        _ => {
                            return Err(MapperError::UnsupportedContent {
                                role: "User".to_string(),
                                protocol: "OpenAI".to_string(),
                            });
                        }
                    }
                }

                Ok(vec![ChatCompletionMessage::User {
                    content: UserContent::Parts(parts),
                    name: msg.name,
                }])
            }
            Role::Assistant => {
                let mut content: Option<String> = None;
                let mut tool_calls = Vec::new();
                let mut refusal = None;

                for part in msg.content {
                    match part {
                        ContentPart::Json(value) => match content {
                            Some(ref mut c) => {
                                c.push_str("\n\n");
                                c.push_str(
                                    &serde_json::to_string(&value)
                                        .map_err(MapperError::JsonError)?,
                                );
                            }
                            None => {
                                content = Some(
                                    serde_json::to_string(&value)
                                        .map_err(MapperError::JsonError)?,
                                )
                            }
                        },
                        ContentPart::Text { text, signature: _ } => match content {
                            Some(ref mut c) => {
                                c.push_str("\n\n");
                                c.push_str(&text);
                            }
                            None => content = Some(text),
                        },
                        ContentPart::ToolCall(tc) => {
                            if tc.name.is_empty() {
                                return Err(MapperError::MissingField {
                                    field: "tool_call.function.name".to_string(),
                                });
                            }
                            let arguments = match &tc.arguments {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            tool_calls.push(OpenAIToolCall {
                                id: tc.id,
                                r#type: "function".into(),
                                function: Some(ToolCallFunction {
                                    name: tc.name,
                                    arguments: arguments.into(),
                                }),
                                custom: None,
                            });
                        }
                        ContentPart::Refusal { refusal: r } => {
                            refusal = Some(r);
                        }
                        _ => {
                            return Err(MapperError::UnsupportedContent {
                                role: "Assistant".to_string(),
                                protocol: "OpenAI".to_string(),
                            });
                        }
                    }
                }

                let tool_calls = if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                };

                let content = content.filter(|c| !c.is_empty());

                Ok(vec![ChatCompletionMessage::Assistant {
                    content: content.map(Into::into),
                    name: msg.name,
                    tool_calls,
                    refusal,
                    audio: None,
                    function_call: None,
                }])
            }
            Role::Tool => {
                if msg.content.is_empty() {
                    return Err(MapperError::MissingToolResult);
                }

                let mut tool_messages = Vec::with_capacity(msg.content.len());
                for part in msg.content {
                    match part {
                        ContentPart::ToolResult(res) => {
                            let content_str = if res.content.len() == 1 {
                                match &res.content[0] {
                                    ContentPart::Text { text, signature: _ } => text.clone(),
                                    ContentPart::Json(value) => serde_json::to_string(value)
                                        .map_err(MapperError::JsonError)?,
                                    _ => serde_json::to_string(&res.content)?,
                                }
                            } else {
                                serde_json::to_string(&res.content)?
                            };

                            tool_messages.push(ChatCompletionMessage::Tool {
                                content: content_str.into(),
                                tool_call_id: res.tool_call_id,
                            });
                        }
                        _ => return Err(MapperError::MissingToolResult),
                    }
                }

                Ok(tool_messages)
            }
        }
    }

    fn convert_image_openai(image: Image) -> Result<ImageUrl, MapperError> {
        let url = match image.source {
            ImageSource::Url { url } => url,
            ImageSource::Base64 { data } => {
                if let Some(media_type) = image.media_type {
                    format!("data:{};base64,{}", media_type, data).into()
                } else {
                    return Err(MapperError::InvalidMediaType);
                }
            }
        };

        Ok(ImageUrl {
            url,
            detail: image.detail,
        })
    }

    fn convert_audio_openai(audio: Audio) -> Result<InputAudio, MapperError> {
        Ok(InputAudio {
            data: audio.data,
            format: audio.format,
        })
    }

    /// Convert OpenAI ChatCompletionResponse to core Message.
    ///
    /// 将 OpenAI ChatCompletionResponse 转换为核心 Message。
    pub fn to_core_message(resp: ChatCompletionResponse) -> Result<Message, MapperError> {
        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or(MapperError::EmptyResponse)?;
        let msg = choice.message;

        let mut content_parts = Vec::new();

        // 1. Text Content
        if let Some(content) = msg.content {
            content_parts.push(ContentPart::Text {
                text: content,
                signature: None,
            });
        }

        // 2. Tool Calls
        if let Some(tool_calls) = msg.tool_calls {
            for tc in tool_calls {
                if let Some(func) = tc.function {
                    if func.name.is_empty() {
                        return Err(MapperError::MissingField {
                            field: "tool_call.function.name".to_string(),
                        });
                    }
                    content_parts.push(ContentPart::ToolCall(ToolCall {
                        id: tc.id,
                        name: func.name,
                        arguments: serde_json::from_str(&func.arguments)
                            .map_err(MapperError::JsonError)?,
                        signature: None,
                    }));
                }
            }
        }

        // 3. Refusal
        if let Some(refusal) = msg.refusal {
            content_parts.push(ContentPart::Refusal { refusal });
        }

        // 4. Audio
        if let Some(audio) = msg.audio {
            content_parts.push(ContentPart::Audio(crate::message::Audio {
                data: audio.data,
                format: "wav".into(), // OpenAI typically uses wav/pcm, defaulting here.
            }));
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }

    /// Convert `ToolDefinition` to `OpenAIChatCompletionsTool`.
    ///
    /// 将 `ToolDefinition` 转换为 `OpenAIChatCompletionsTool`。
    pub fn tool_to_openai(tool: &ToolDefinition) -> OpenAIChatCompletionsTool {
        let parameters = tool
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok());

        OpenAIChatCompletionsTool {
            r#type: "function".into(),
            function: Some(OpenAIFunctionDefinition {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters,
                strict: tool.function.strict,
            }),
            custom: None,
        }
    }

    /// Convert `ToolChoice` to `OpenAIChatCompletionsToolChoice`.
    ///
    /// 将 `ToolChoice` 转换为 `OpenAIChatCompletionsToolChoice`。
    pub fn tool_choice_to_openai(choice: &ToolChoice) -> OpenAIChatCompletionsToolChoice {
        match choice {
            ToolChoice::None => OpenAIChatCompletionsToolChoice::String("none".into()),
            ToolChoice::Auto => OpenAIChatCompletionsToolChoice::String("auto".into()),
            ToolChoice::Required => OpenAIChatCompletionsToolChoice::String("required".into()),
            ToolChoice::Function { name } => {
                OpenAIChatCompletionsToolChoice::Named(OpenAIChatCompletionsToolChoiceNamed {
                    r#type: "function".into(),
                    function: Some(OpenAIChatCompletionsToolChoiceFunction { name: name.clone() }),
                    custom: None,
                })
            }
        }
    }

    /// Convert `OpenAIChatCompletionsTool` to `ToolDefinition`.
    ///
    /// 将 `OpenAIChatCompletionsTool` 转换为 `ToolDefinition`。
    pub fn tool_from_openai(
        value: OpenAIChatCompletionsTool,
    ) -> Result<ToolDefinition, MapperError> {
        if value.r#type != "function" || value.function.is_none() {
            return Err(MapperError::UnsupportedContent {
                role: "Tool".to_string(),
                protocol: format!("OpenAI tool type: {}", value.r#type),
            });
        }
        let func = value.function.unwrap();
        let parameters = match func.parameters {
            Some(v) => Some(serde_json::from_value(v).map_err(MapperError::JsonError)?),
            None => None,
        };
        Ok(ToolDefinition {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: func.name,
                description: func.description,
                parameters,
                strict: func.strict,
            },
        })
    }

    /// Convert `OpenAIChatCompletionsToolChoice` to `ToolChoice`.
    ///
    /// 将 `OpenAIChatCompletionsToolChoice` 转换为 `ToolChoice`。
    pub fn tool_choice_from_openai(value: OpenAIChatCompletionsToolChoice) -> ToolChoice {
        match value {
            OpenAIChatCompletionsToolChoice::String(s) => match s.as_ref() {
                "none" => ToolChoice::None,
                "required" => ToolChoice::Required,
                _ => ToolChoice::Auto,
            },
            OpenAIChatCompletionsToolChoice::Named(named) => {
                if let Some(func) = named.function {
                    ToolChoice::Function { name: func.name }
                } else if let Some(custom) = named.custom {
                    ToolChoice::Function { name: custom.name }
                } else {
                    ToolChoice::Auto
                }
            }
            OpenAIChatCompletionsToolChoice::Allowed(_) => ToolChoice::Auto,
        }
    }
}

impl TryFrom<Message> for Vec<ChatCompletionMessage> {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        OpenAIChatCompletionMapper::from_core_message(msg)
    }
}

impl TryFrom<ChatCompletionResponse> for Message {
    type Error = MapperError;

    fn try_from(resp: ChatCompletionResponse) -> Result<Self, Self::Error> {
        OpenAIChatCompletionMapper::to_core_message(resp)
    }
}

impl TryFrom<&ToolDefinition> for OpenAIChatCompletionsTool {
    type Error = MapperError;

    fn try_from(tool: &ToolDefinition) -> Result<Self, Self::Error> {
        Ok(OpenAIChatCompletionMapper::tool_to_openai(tool))
    }
}

impl TryFrom<&ToolChoice> for OpenAIChatCompletionsToolChoice {
    type Error = MapperError;

    fn try_from(choice: &ToolChoice) -> Result<Self, Self::Error> {
        Ok(OpenAIChatCompletionMapper::tool_choice_to_openai(choice))
    }
}

impl TryFrom<OpenAIChatCompletionsTool> for ToolDefinition {
    type Error = MapperError;

    fn try_from(value: OpenAIChatCompletionsTool) -> Result<Self, Self::Error> {
        OpenAIChatCompletionMapper::tool_from_openai(value)
    }
}

impl TryFrom<OpenAIChatCompletionsToolChoice> for ToolChoice {
    type Error = MapperError;

    fn try_from(value: OpenAIChatCompletionsToolChoice) -> Result<Self, Self::Error> {
        Ok(OpenAIChatCompletionMapper::tool_choice_from_openai(value))
    }
}

/// OpenAI Chat Completions Raw Conversation State.
///
/// OpenAI Chat Completions 原始对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletionsConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<StaticRefStr>,

    /// Raw Message list.
    ///
    /// 原始消息列表。
    pub messages: Vec<ChatCompletionMessage>,

    /// Available raw tools.
    ///
    /// 可用原始工具列表。
    pub tools: Vec<OpenAIChatCompletionsTool>,

    /// Raw Tool choice preference.
    ///
    /// 原始工具选择偏好。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<OpenAIChatCompletionsToolChoice>,
}

impl ConversationStateTrait for ChatCompletionsConversationState {}

impl TryFrom<ConversationState> for ChatCompletionsConversationState {
    type Error = MapperError;

    fn try_from(state: ConversationState) -> Result<Self, Self::Error> {
        let mut messages = Vec::new();
        for msg in state.messages {
            messages.extend(OpenAIChatCompletionMapper::from_core_message(msg)?);
        }
        let tools = state
            .tools
            .iter()
            .map(OpenAIChatCompletionMapper::tool_to_openai)
            .collect();
        let tool_choice = state
            .tool_choice
            .as_ref()
            .map(OpenAIChatCompletionMapper::tool_choice_to_openai);

        Ok(ChatCompletionsConversationState {
            system_prompt: state.system_prompt,
            messages,
            tools,
            tool_choice,
        })
    }
}

/// A stateful mapper for OpenAI streaming responses.
///
/// 用于 OpenAI 流式响应的有状态映射器。
pub struct OpenAIStreamMapper;

impl OpenAIStreamMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map_response(&mut self, chunk: OpenAIStreamChunk) -> Result<DeltaMessage, MapperError> {
        // Usage might be present at the end of the stream (stream_options)
        let usage = chunk.usage.map(|u| crate::message::Usage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            reasoning_tokens: u
                .completion_tokens_details
                .as_ref()
                .and_then(|d| d.reasoning_tokens),
            cached_input_tokens: u
                .prompt_tokens_details
                .as_ref()
                .and_then(|d| d.cached_tokens),
            cached_output_tokens: None,
            tool_use_tokens: None,
        });

        let choice = match chunk.choices.into_iter().next() {
            Some(choice) => choice,
            None => {
                // If there are no choices but there is usage, it's the final usage chunk
                if usage.is_some() {
                    return Ok(DeltaMessage {
                        role: None,
                        content: None,
                        name: None,
                        finish_reason: None,
                        usage,
                    });
                }
                // Otherwise it might be an empty chunk or error, but let's return empty Delta
                return Ok(DeltaMessage::default());
            }
        };
        let delta = choice.delta;

        let role = match delta.role.as_deref() {
            Some("user") => Some(Role::User),
            Some("assistant") => Some(Role::Assistant),
            Some("tool") => Some(Role::Tool),
            _ => None,
        };

        let finish_reason = choice.finish_reason.map(|r| match r.as_ref() {
            "stop" => crate::message::FinishReason::Stop,
            "length" => crate::message::FinishReason::Length,
            "tool_calls" | "function_call" => crate::message::FinishReason::ToolCalls,
            "content_filter" => crate::message::FinishReason::ContentFilter,
            _ => crate::message::FinishReason::Other(r),
        });

        let mut content_parts = Vec::new();

        if let Some(content) = delta.content {
            content_parts.push(DeltaContentPart::Text {
                index: 0,
                text: content,
                signature: None,
            });
        }

        if let Some(reasoning) = delta.reasoning_content {
            content_parts.push(DeltaContentPart::Reasoning {
                index: 0,
                text: reasoning,
                signature: None,
            });
        }

        if let Some(refusal) = delta.refusal {
            content_parts.push(DeltaContentPart::Refusal { refusal });
        }

        if let Some(tool_calls) = delta.tool_calls {
            for tc in tool_calls {
                content_parts.push(DeltaContentPart::ToolCall(DeltaToolCall {
                    index: tc.index,
                    id: tc.id,
                    r#type: tc.r#type,
                    function: tc.function.map(|f| DeltaFunction {
                        name: f.name,
                        arguments: f.arguments,
                    }),
                    signature: None,
                }));
            }
        }

        let content = if content_parts.is_empty() {
            None
        } else {
            Some(content_parts)
        };

        Ok(DeltaMessage {
            role,
            content,
            name: None,
            finish_reason,
            usage,
        })
    }
}

impl Default for OpenAIStreamMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use oxide_llm_proto::openai::v1::chat_completions::ToolContent;

    use super::*;
    use crate::tool::ToolResult;

    #[test]
    fn test_openai_chat_completion_mapper_multiple_tool_results() {
        let msg = Message {
            role: Role::Tool,
            content: vec![
                ContentPart::ToolResult(ToolResult {
                    tool_call_id: "call_1".into(),
                    name: "get_weather".into(),
                    content: vec![ContentPart::Text {
                        text: "Sunny".into(),
                        signature: None,
                    }],
                    is_error: false,
                    signature: None,
                }),
                ContentPart::ToolResult(ToolResult {
                    tool_call_id: "call_2".into(),
                    name: "get_stock_price".into(),
                    content: vec![ContentPart::Text {
                        text: "$240.00".into(),
                        signature: None,
                    }],
                    is_error: false,
                    signature: None,
                }),
            ],
            name: None,
        };

        let mapped = OpenAIChatCompletionMapper::from_core_message(msg).unwrap();
        assert_eq!(mapped.len(), 2);

        if let ChatCompletionMessage::Tool {
            content,
            tool_call_id,
        } = &mapped[0]
        {
            assert_eq!(content, &ToolContent::Text("Sunny".into()));
            assert_eq!(tool_call_id, "call_1");
        } else {
            panic!("Expected ChatCompletionMessage::Tool");
        }

        if let ChatCompletionMessage::Tool {
            content,
            tool_call_id,
        } = &mapped[1]
        {
            assert_eq!(content, &ToolContent::Text("$240.00".into()));
            assert_eq!(tool_call_id, "call_2");
        } else {
            panic!("Expected ChatCompletionMessage::Tool");
        }
    }

    #[test]
    fn test_openai_chat_completion_mapper_assistant_tool_call_arguments() {
        let raw_args = r#"{"location":"Tokyo"}"#;
        let msg = Message {
            role: Role::Assistant,
            content: vec![ContentPart::ToolCall(ToolCall {
                id: "call_123".into(),
                name: "get_weather".into(),
                arguments: serde_json::Value::String(raw_args.to_string()),
                signature: None,
            })],
            name: None,
        };

        let mapped = OpenAIChatCompletionMapper::from_core_message(msg).unwrap();
        assert_eq!(mapped.len(), 1);

        if let ChatCompletionMessage::Assistant { tool_calls, .. } = &mapped[0] {
            let calls = tool_calls.as_ref().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(
                calls[0].function.as_ref().unwrap().arguments.as_str(),
                raw_args
            );
        } else {
            panic!("Expected ChatCompletionMessage::Assistant");
        }
    }

    #[test]
    fn test_openai_chat_completion_mapper_empty_tool_name_error() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![ContentPart::ToolCall(ToolCall {
                id: "call_123".into(),
                name: "".into(),
                arguments: serde_json::Value::Null,
                signature: None,
            })],
            name: None,
        };

        let err = OpenAIChatCompletionMapper::from_core_message(msg).unwrap_err();
        assert!(
            matches!(err, MapperError::MissingField { ref field } if field == "tool_call.function.name")
        );
    }

    #[test]
    fn test_openai_chat_completion_stream_usage() {
        let mut mapper = OpenAIStreamMapper::new();
        let chunk_json = serde_json::json!({
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150,
                "completion_tokens_details": {
                    "reasoning_tokens": 30
                },
                "prompt_tokens_details": {
                    "cached_tokens": 20
                }
            }
        });

        let chunk: OpenAIStreamChunk = serde_json::from_value(chunk_json).unwrap();
        let delta = mapper.map_response(chunk).unwrap();
        let usage = delta.usage.expect("Usage should be present");
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.reasoning_tokens, Some(30));
        assert_eq!(usage.cached_input_tokens, Some(20));
    }
}
