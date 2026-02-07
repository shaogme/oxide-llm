use crate::mapper::MapperError;
use crate::message::{Audio, ContentPart, Image, ImageSource, Message, Role};
use crate::message::{DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall};
use crate::tool::ToolCall;
use oxide_llm_proto::openai::v1::chat_completions::chunk::ChatCompletionChunk;
use oxide_llm_proto::openai::v1::chat_completions::request::{
    ChatCompletionMessage, ContentPart as OpenAIContentPart, ImageUrl, InputAudio, UserContent,
};
use oxide_llm_proto::openai::v1::chat_completions::response::ChatCompletionResponse;
use oxide_llm_proto::openai::v1::{ToolCall as OpenAIToolCall, ToolCallFunction};

/// Convert core Message to OpenAI ChatCompletionMessage.
///
/// 将核心 Message 转换为 OpenAI ChatCompletionMessage。
impl TryFrom<Message> for ChatCompletionMessage {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        match msg.role {
            Role::User => {
                if msg.content.len() == 1 {
                    if let ContentPart::Text { text } = &msg.content[0] {
                        return Ok(ChatCompletionMessage::User {
                            content: UserContent::Text(text.clone()),
                            name: msg.name,
                        });
                    }
                }

                let mut parts = Vec::new();
                for part in msg.content {
                    match part {
                        ContentPart::Text { text } => {
                            parts.push(OpenAIContentPart::Text { text });
                        }
                        ContentPart::Image(image) => {
                            parts.push(OpenAIContentPart::ImageUrl {
                                image_url: convert_image_openai(image)?,
                            });
                        }
                        ContentPart::Audio(audio) => {
                            parts.push(OpenAIContentPart::InputAudio {
                                input_audio: convert_audio_openai(audio)?,
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
                        ContentPart::Text { text } => match content {
                            Some(ref mut c) => {
                                c.push_str("\n\n");
                                c.push_str(&text);
                            }
                            None => content = Some(text),
                        },
                        ContentPart::ToolCall(tc) => {
                            tool_calls.push(OpenAIToolCall {
                                id: tc.id,
                                r#type: "function".to_string(),
                                function: ToolCallFunction {
                                    name: tc.name,
                                    arguments: tc.arguments.to_string(),
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
                    content,
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
                                ContentPart::Text { text } => text.clone(),
                                ContentPart::Json(value) => {
                                    serde_json::to_string(value).map_err(MapperError::JsonError)?
                                }
                                _ => serde_json::to_string(&res.content)?,
                            }
                        } else {
                            serde_json::to_string(&res.content)?
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
}

fn convert_image_openai(image: Image) -> Result<ImageUrl, MapperError> {
    let url = match image.source {
        ImageSource::Url { url } => url,
        ImageSource::Base64 { data } => {
            if let Some(media_type) = image.media_type {
                format!("data:{};base64,{}", media_type, data)
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
impl TryFrom<ChatCompletionResponse> for Message {
    type Error = MapperError;

    fn try_from(resp: ChatCompletionResponse) -> Result<Self, Self::Error> {
        let choice = resp.choices.first().ok_or(MapperError::EmptyResponse)?;
        let msg = &choice.message;

        let mut content_parts = Vec::new();

        // 1. Text Content
        if let Some(content) = &msg.content {
            content_parts.push(ContentPart::Text {
                text: content.clone(),
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
                format: "wav".to_string(), // OpenAI typically uses wav/pcm, defaulting here.
            }));
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }
}

/// Convert OpenAI ChatCompletionChunk to core DeltaMessage.
///
/// 将 OpenAI ChatCompletionChunk 转换为核心 DeltaMessage。
impl TryFrom<ChatCompletionChunk> for DeltaMessage {
    type Error = MapperError;

    fn try_from(chunk: ChatCompletionChunk) -> Result<Self, Self::Error> {
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

        let finish_reason = choice.finish_reason.as_ref().map(|r| match r.as_str() {
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
