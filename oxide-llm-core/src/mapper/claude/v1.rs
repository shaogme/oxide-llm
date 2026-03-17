use crate::mapper::MapperError;
use crate::message::{ContentPart, ImageSource, Message, Role};
use crate::message::{DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall};
use crate::tool::ToolCall;
use oxide_llm_proto::claude::v1::messages::request::Message as ClaudeMessage;
use oxide_llm_proto::claude::v1::messages::response::MessagesResponse;
use oxide_llm_proto::claude::v1::messages::{
    Content as ClaudeContent, ContentBlock, ImageBlock, ImageSource as ClaudeImageSource,
    Role as ClaudeRole, TextBlock, ThinkingBlock, ToolResultBlock, ToolResultContent, ToolUseBlock,
};

/// Convert core Message to Claude Message.
///
/// 将核心 Message 转换为 Claude Message。
impl TryFrom<Message> for ClaudeMessage {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        match msg.role {
            Role::User => {
                let blocks = convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::User,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
            Role::Assistant => {
                let blocks = convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::Assistant,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
            Role::Tool => {
                // In Claude, tool results are sent as User messages with ToolResult blocks
                let blocks = convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::User,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
        }
    }
}

fn convert_content_to_claude_blocks(
    parts: Vec<ContentPart>,
) -> Result<Vec<ContentBlock>, MapperError> {
    let mut blocks = Vec::new();
    for part in parts {
        match part {
            ContentPart::Text { text, signature: _ } => {
                blocks.push(ContentBlock::Text(TextBlock {
                    text,
                    cache_control: None, // Cache control not supported in generic mapper yet
                    citations: None,
                }));
            }
            ContentPart::Image(image) => {
                let source = match image.source {
                    ImageSource::Base64 { data } => ClaudeImageSource::Base64 {
                        r#type: "base64".to_string(),
                        media_type: image.media_type.ok_or(MapperError::InvalidMediaType)?,
                        data,
                    },
                    ImageSource::Url { url } => ClaudeImageSource::Url {
                        r#type: "url".to_string(),
                        url,
                    },
                };
                blocks.push(ContentBlock::Image(ImageBlock {
                    source,
                    cache_control: None,
                }));
            }
            ContentPart::ToolCall(tc) => {
                blocks.push(ContentBlock::ToolUse(ToolUseBlock {
                    id: tc.id,
                    name: tc.name,
                    input: tc.arguments,
                    cache_control: None,
                }));
            }
            ContentPart::ToolResult(tr) => {
                let content = if tr.content.len() == 1 {
                    match &tr.content[0] {
                        ContentPart::Text { text, signature: _ } => ToolResultContent::Text(text.clone()),
                        ContentPart::Json(value) => {
                            let text =
                                serde_json::to_string(value).map_err(MapperError::JsonError)?;
                            ToolResultContent::Text(text)
                        }
                        _ => {
                            ToolResultContent::Blocks(convert_content_to_claude_blocks(tr.content)?)
                        }
                    }
                } else {
                    ToolResultContent::Blocks(convert_content_to_claude_blocks(tr.content)?)
                };

                blocks.push(ContentBlock::ToolResult(ToolResultBlock {
                    tool_use_id: tr.tool_call_id,
                    content,
                    is_error: if tr.is_error { Some(true) } else { None },
                    cache_control: None,
                }));
            }
            ContentPart::Json(value) => {
                let text = serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                blocks.push(ContentBlock::Text(TextBlock {
                    text,
                    cache_control: None,
                    citations: None,
                }));
            }
            ContentPart::Reasoning { text, signature } => {
                blocks.push(ContentBlock::Thinking(ThinkingBlock {
                    thinking: text,
                    signature: signature.unwrap_or_default(),
                }));
            }
            _ => {
                // Audio, Refusal not directly supported in Claude basic blocks or mapped differently
                return Err(MapperError::UnsupportedContent {
                    role: "Any".to_string(),
                    protocol: "Claude".to_string(),
                });
            }
        }
    }
    Ok(blocks)
}

/// Convert Claude MessagesResponse to core Message.
///
/// 将 Claude MessagesResponse 转换为核心 Message。
impl TryFrom<MessagesResponse> for Message {
    type Error = MapperError;

    fn try_from(resp: MessagesResponse) -> Result<Self, Self::Error> {
        let mut content_parts = Vec::new();

        for block in resp.content {
            match block {
                ContentBlock::Text(text_block) => {
                    content_parts.push(ContentPart::Text {
                        text: text_block.text,
                        signature: None,
                    });
                }
                ContentBlock::ToolUse(tool_use) => {
                    content_parts.push(ContentPart::ToolCall(ToolCall {
                        id: tool_use.id,
                        name: tool_use.name,
                        arguments: tool_use.input,
                        signature: None,
                    }));
                }
                // Skip other blocks like Thinking for now as Core doesn't support them explicitly yet
                _ => {}
            }
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }
}

/// Convert Claude MessageStreamEvent to core DeltaMessage.
///
/// 将 Claude MessageStreamEvent 转换为核心 DeltaMessage。
impl TryFrom<oxide_llm_proto::claude::v1::messages::chunk::MessageStreamEvent> for DeltaMessage {
    type Error = MapperError;

    fn try_from(
        event: oxide_llm_proto::claude::v1::messages::chunk::MessageStreamEvent,
    ) -> Result<Self, Self::Error> {
        use oxide_llm_proto::claude::v1::messages::chunk::{
            ChunkContentBlock, ChunkContentBlockDelta, MessageStreamEvent,
        };
        use oxide_llm_proto::claude::v1::messages::response::StopReason;

        match event {
            MessageStreamEvent::MessageStart { message } => {
                let role = match message.role {
                    ClaudeRole::User => Role::User,
                    ClaudeRole::Assistant => Role::Assistant,
                };
                let usage = crate::message::Usage {
                    input_tokens: message.usage.input_tokens,
                    output_tokens: message.usage.output_tokens,
                    total_tokens: message.usage.input_tokens + message.usage.output_tokens,
                };

                Ok(DeltaMessage {
                    role: Some(role),
                    content: None,
                    name: None,
                    finish_reason: None,
                    usage: Some(usage),
                })
            }
            MessageStreamEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                let part = match content_block {
                    ChunkContentBlock::ToolUse(tool_use) => {
                        Some(DeltaContentPart::ToolCall(DeltaToolCall {
                            index,
                            id: Some(tool_use.id),
                            r#type: Some("function".to_string()),
                            function: Some(DeltaFunction {
                                name: Some(tool_use.name),
                                arguments: None, // Arguments come in Delta
                            }),
                            signature: None,
                        }))
                    }
                    ChunkContentBlock::Thinking(t) => {
                        // Start of thinking block
                        Some(DeltaContentPart::Reasoning {
                            index,
                            text: t.thinking,
                            signature: None,
                        })
                    }
                    ChunkContentBlock::RedactedThinking(t) => {
                        Some(DeltaContentPart::Reasoning {
                            index,
                            text: String::new(),
                            signature: Some(t.data), // Encrypted/Redacted data as signature
                        })
                    }
                    ChunkContentBlock::ServerToolUse(stu) => {
                        Some(DeltaContentPart::ToolCall(DeltaToolCall {
                            index,
                            id: Some(stu.id),
                            r#type: Some("server_function".to_string()),
                            function: Some(DeltaFunction {
                                name: Some(stu.name),
                                arguments: Some(stu.input.to_string()),
                            }),
                            signature: None,
                        }))
                    }
                    ChunkContentBlock::SearchResult(sr) => {
                        let mut text = format!("[Search Result: {}]\n", sr.title);
                        for block in sr.content {
                            text.push_str(&block.text);
                            text.push('\n');
                        }
                        Some(DeltaContentPart::Text {
                            index,
                            text,
                            signature: None,
                        })
                    }
                    ChunkContentBlock::WebSearchToolResult(wstr) => {
                        let mut text = String::from("[Web Search Results]\n");
                        for item in wstr.content {
                            if let oxide_llm_proto::claude::v1::messages::WebSearchResultItem::WebSearchResult(res) = item {
                                     use std::fmt::Write;
                                     let _ = writeln!(text, "- {} ({}) : {}", res.title, res.url, res.encrypted_content);
                                 }
                        }
                        Some(DeltaContentPart::Text {
                            index,
                            text,
                            signature: None,
                        })
                    }
                    ChunkContentBlock::Text(t) => {
                        if !t.text.is_empty() {
                            Some(DeltaContentPart::Text {
                                index,
                                text: t.text,
                                signature: None,
                            })
                        } else {
                            None
                        }
                    }
                    ChunkContentBlock::Image(_) => {
                        // Current DeltaContentPart does not support Image.
                        // Images in streaming responses are rare or typically static references.
                        None
                    }
                };

                Ok(DeltaMessage {
                    role: None,
                    content: part.map(|p| vec![p]),
                    name: None,
                    finish_reason: None,
                    usage: None,
                })
            }
            MessageStreamEvent::ContentBlockDelta { index, delta } => {
                let part = match delta {
                    ChunkContentBlockDelta::TextDelta { text } => {
                        Some(DeltaContentPart::Text {
                            index,
                            text,
                            signature: None,
                        })
                    }
                    ChunkContentBlockDelta::InputJsonDelta { partial_json } => {
                        Some(DeltaContentPart::ToolCall(DeltaToolCall {
                            index,
                            id: None,
                            r#type: None,
                            function: Some(DeltaFunction {
                                name: None,
                                arguments: Some(partial_json),
                            }),
                            signature: None,
                        }))
                    }
                    ChunkContentBlockDelta::ThinkingDelta { thinking } => {
                        Some(DeltaContentPart::Reasoning {
                            index,
                            text: thinking,
                            signature: None,
                        })
                    }
                    ChunkContentBlockDelta::SignatureDelta { signature } => {
                        Some(DeltaContentPart::Reasoning {
                            index,
                            text: String::new(),
                            signature: Some(signature),
                        })
                    }
                };

                Ok(DeltaMessage {
                    role: None,
                    content: part.map(|p| vec![p]),
                    name: None,
                    finish_reason: None,
                    usage: None,
                })
            }
            MessageStreamEvent::ContentBlockStop { .. } => {
                // Usually no data needed for stop
                Err(MapperError::IgnoredEvent {
                    event_type: "content_block_stop".to_string(),
                })
            }
            MessageStreamEvent::MessageDelta { delta, usage } => {
                let finish_reason = delta.stop_reason.map(|r| match r {
                    StopReason::EndTurn => crate::message::FinishReason::Stop,
                    StopReason::MaxTokens => crate::message::FinishReason::Length,
                    StopReason::StopSequence => crate::message::FinishReason::Stop, // or Other
                    StopReason::ToolUse => crate::message::FinishReason::ToolCalls,
                    _ => crate::message::FinishReason::Other(format!("{:?}", r)),
                });

                let usage = crate::message::Usage {
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    total_tokens: usage.input_tokens + usage.output_tokens,
                };

                Ok(DeltaMessage {
                    role: None,
                    content: None,
                    name: None,
                    finish_reason,
                    usage: Some(usage),
                })
            }
            MessageStreamEvent::MessageStop => Err(MapperError::IgnoredEvent {
                event_type: "message_stop".to_string(),
            }),
            MessageStreamEvent::Ping => Err(MapperError::IgnoredEvent {
                event_type: "ping".to_string(),
            }),
            MessageStreamEvent::Error { error } => Err(MapperError::UnsupportedContent {
                role: "System".to_string(),
                protocol: format!("Claude Error: {}", error.message),
            }),
        }
    }
}
