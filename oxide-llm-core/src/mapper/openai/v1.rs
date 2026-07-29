use crate::mapper::MapperError;
use crate::message::{Audio, ContentPart, Image, ImageSource, Message, Role};
use crate::message::{DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall};
use crate::tool::ToolCall;
use oxide_llm_proto::openai::v1::chat_completions::chunk::ChatCompletionChunk as OpenAIStreamChunk;
use oxide_llm_proto::openai::v1::chat_completions::request::{
    ChatCompletionMessage, ContentPart as OpenAIContentPart, ImageUrl, InputAudio, UserContent,
};
use oxide_llm_proto::openai::v1::chat_completions::response::ChatCompletionResponse;
use oxide_llm_proto::openai::v1::{ToolCall as OpenAIToolCall, ToolCallFunction};

/// Mapper for OpenAI protocol.
///
/// OpenAI 协议映射器。
pub struct OpenAIMapper;

impl OpenAIMapper {
    /// Convert core Message to OpenAI ChatCompletionMessage.
    ///
    /// 将核心 Message 转换为 OpenAI ChatCompletionMessage。
    pub fn from_core_message(msg: Message) -> Result<ChatCompletionMessage, MapperError> {
        match msg.role {
            Role::User => {
                if msg.content.len() == 1
                    && let ContentPart::Text { text, signature: _ } = &msg.content[0]
                {
                    return Ok(ChatCompletionMessage::User {
                        content: UserContent::Text(text.clone()),
                        name: msg.name,
                    });
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
                            parts.push(OpenAIContentPart::Text { text: text.into() });
                        }
                        _ => {
                            return Err(MapperError::UnsupportedContent {
                                role: "User".to_string(),
                                protocol: "OpenAI".to_string(),
                            });
                        }
                    }
                }

                Ok(ChatCompletionMessage::User {
                    content: UserContent::Parts(parts),
                    name: msg.name,
                })
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
                            None => content = Some(text.into()),
                        },
                        ContentPart::ToolCall(tc) => {
                            tool_calls.push(OpenAIToolCall {
                                id: tc.id,
                                r#type: "function".into(),
                                function: ToolCallFunction {
                                    name: tc.name,
                                    arguments: tc.arguments.to_string().into(),
                                },
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

                Ok(ChatCompletionMessage::Assistant {
                    content: content.map(Into::into),
                    name: msg.name,
                    tool_calls,
                    refusal,
                })
            }
            Role::Tool => {
                if msg.content.len() != 1 {
                    return Err(MapperError::InvalidOpenAIToolMessage);
                }

                match &msg.content[0] {
                    ContentPart::ToolResult(res) => {
                        let content_str = if res.content.len() == 1 {
                            match &res.content[0] {
                                ContentPart::Text { text, signature: _ } => text.clone(),
                                ContentPart::Json(value) => serde_json::to_string(value)
                                    .map_err(MapperError::JsonError)?
                                    .into(),
                                _ => serde_json::to_string(&res.content)?.into(),
                            }
                        } else {
                            serde_json::to_string(&res.content)?.into()
                        };

                        Ok(ChatCompletionMessage::Tool {
                            content: content_str,
                            tool_call_id: res.tool_call_id.clone(),
                        })
                    }
                    _ => Err(MapperError::MissingToolResult),
                }
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
        let choice = resp.choices.first().ok_or(MapperError::EmptyResponse)?;
        let msg = &choice.message;

        let mut content_parts = Vec::new();

        // 1. Text Content
        if let Some(content) = &msg.content {
            content_parts.push(ContentPart::Text {
                text: content.clone(),
                signature: None,
            });
        }

        // 2. Tool Calls
        if let Some(tool_calls) = &msg.tool_calls {
            for tc in tool_calls {
                content_parts.push(ContentPart::ToolCall(ToolCall {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: serde_json::from_str(&tc.function.arguments)
                        .map_err(MapperError::JsonError)?,
                    signature: None,
                }));
            }
        }

        // 3. Refusal
        if let Some(refusal) = &msg.refusal {
            content_parts.push(ContentPart::Refusal {
                refusal: refusal.clone(),
            });
        }

        // 4. Audio
        if let Some(audio) = &msg.audio {
            content_parts.push(ContentPart::Audio(crate::message::Audio {
                data: audio.data.clone(),
                format: "wav".into(), // OpenAI typically uses wav/pcm, defaulting here.
            }));
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
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
        });

        if chunk.choices.is_empty() {
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

        let choice = &chunk.choices[0];
        let delta = &choice.delta;

        let role = match delta.role.as_deref() {
            Some("user") => Some(Role::User),
            Some("assistant") => Some(Role::Assistant),
            Some("tool") => Some(Role::Tool),
            _ => None,
        };

        let finish_reason = choice.finish_reason.as_ref().map(|r| match r.as_ref() {
            "stop" => crate::message::FinishReason::Stop,
            "length" => crate::message::FinishReason::Length,
            "tool_calls" | "function_call" => crate::message::FinishReason::ToolCalls,
            "content_filter" => crate::message::FinishReason::ContentFilter,
            _ => crate::message::FinishReason::Other(r.clone()),
        });

        let mut content_parts = Vec::new();

        if let Some(content) = &delta.content {
            content_parts.push(DeltaContentPart::Text {
                index: 0,
                text: content.clone(),
                signature: None,
            });
        }

        if let Some(reasoning) = &delta.reasoning_content {
            content_parts.push(DeltaContentPart::Reasoning {
                index: 0,
                text: reasoning.clone(),
                signature: None,
            });
        }

        if let Some(refusal) = &delta.refusal {
            content_parts.push(DeltaContentPart::Refusal {
                refusal: refusal.clone(),
            });
        }

        if let Some(tool_calls) = &delta.tool_calls {
            for tc in tool_calls {
                content_parts.push(DeltaContentPart::ToolCall(DeltaToolCall {
                    index: tc.index,
                    id: tc.id.clone(),
                    r#type: tc.r#type.clone(),
                    function: tc.function.as_ref().map(|f| DeltaFunction {
                        name: f.name.clone(),
                        arguments: f.arguments.clone(),
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
