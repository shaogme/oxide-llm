use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use oxide_llm_proto::claude::v1::messages::{
    Content as ClaudeContent, ContentBlock, CustomTool as ClaudeCustomTool, DocumentBlock,
    DocumentSource as ClaudeDocumentSource, ImageBlock, ImageSource as ClaudeImageSource,
    Message as ClaudeMessage, MessageStreamEvent as ClaudeStreamEvent, MessagesResponse,
    Role as ClaudeRole, TextBlock, ThinkingBlock, Tool as ClaudeTool,
    ToolChoice as ClaudeToolChoice, ToolResultBlock, ToolResultContent, ToolUseBlock,
    WebSearchResultItem,
};

use crate::mapper::MapperError;
use crate::message::{
    ContentPart, DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, ImageSource,
    Message, Role,
};
use crate::state::{ConversationState, ConversationStateTrait};
use crate::tool::{FunctionDefinition, ToolCall, ToolChoice, ToolDefinition, ToolResult, ToolType};

/// Mapper for Claude protocol.
///
/// Claude 协议映射器。
pub struct ClaudeMessagesMapper;

impl ClaudeMessagesMapper {
    /// Convert core Message to Claude Message.
    ///
    /// 将核心 Message 转换为 Claude Message。
    pub fn from_core_message(msg: Message) -> Result<ClaudeMessage, MapperError> {
        match msg.role {
            Role::User => {
                let blocks = Self::convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::User,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
            Role::Assistant => {
                let blocks = Self::convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::Assistant,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
            Role::Tool => {
                // In Claude, tool results are sent as User messages with ToolResult blocks
                let blocks = Self::convert_content_to_claude_blocks(msg.content)?;
                Ok(ClaudeMessage {
                    role: ClaudeRole::User,
                    content: ClaudeContent::Blocks(blocks),
                })
            }
        }
    }

    pub fn convert_content_to_claude_blocks(
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
                            r#type: "base64".into(),
                            media_type: image.media_type.ok_or(MapperError::InvalidMediaType)?,
                            data,
                        },
                        ImageSource::Url { url } => ClaudeImageSource::Url {
                            r#type: "url".into(),
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
                        caller: None,
                        cache_control: None,
                    }));
                }
                ContentPart::ToolResult(tr) => {
                    let content = if tr.content.len() == 1 {
                        match &tr.content[0] {
                            ContentPart::Text { text, signature: _ } => {
                                ToolResultContent::Text(text.clone())
                            }
                            ContentPart::Json(value) => {
                                let text =
                                    serde_json::to_string(value).map_err(MapperError::JsonError)?;
                                ToolResultContent::Text(text)
                            }
                            _ => ToolResultContent::Blocks(Self::convert_content_to_claude_blocks(
                                tr.content,
                            )?),
                        }
                    } else {
                        ToolResultContent::Blocks(Self::convert_content_to_claude_blocks(
                            tr.content,
                        )?)
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
                ContentPart::Document(doc) => {
                    let source = match doc.source {
                        ImageSource::Base64 { data } => ClaudeDocumentSource::Base64 {
                            r#type: "base64".into(),
                            media_type: doc.media_type.unwrap_or_else(|| "application/pdf".into()),
                            data,
                        },
                        ImageSource::Url { url } => ClaudeDocumentSource::Url {
                            r#type: "url".into(),
                            url,
                        },
                    };
                    blocks.push(ContentBlock::Document(DocumentBlock {
                        source,
                        cache_control: None,
                        title: None,
                        context: None,
                        citations: None,
                    }));
                }
                ContentPart::Audio(_) | ContentPart::Video(_) | ContentPart::Refusal { .. } => {}
            }
        }
        Ok(blocks)
    }

    /// Convert Claude MessagesResponse to core Message.
    ///
    /// 将 Claude MessagesResponse 转换为核心 Message。
    pub fn to_core_message(resp: MessagesResponse) -> Result<Message, MapperError> {
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
                ContentBlock::ToolResult(tool_result) => {
                    let parts = match tool_result.content {
                        ToolResultContent::Text(text) => vec![ContentPart::Text {
                            text,
                            signature: None,
                        }],
                        ToolResultContent::Blocks(blocks) => {
                            let mut nested_parts = Vec::new();
                            for block in blocks {
                                match block {
                                    ContentBlock::Text(tb) => {
                                        nested_parts.push(ContentPart::Text {
                                            text: tb.text,
                                            signature: None,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            nested_parts
                        }
                    };

                    content_parts.push(ContentPart::ToolResult(ToolResult {
                        tool_call_id: tool_result.tool_use_id,
                        name: "".into(),
                        content: parts,
                        is_error: tool_result.is_error.unwrap_or(false),
                        signature: None,
                    }));
                }
                ContentBlock::Thinking(thinking_block) => {
                    content_parts.push(ContentPart::Reasoning {
                        text: thinking_block.thinking,
                        signature: if thinking_block.signature.is_empty() {
                            None
                        } else {
                            Some(thinking_block.signature.into())
                        },
                    });
                }
                ContentBlock::RedactedThinking(redacted_block) => {
                    content_parts.push(ContentPart::Reasoning {
                        text: redacted_block.data.to_string(),
                        signature: None,
                    });
                }
                ContentBlock::Image(_)
                | ContentBlock::Document(_)
                | ContentBlock::SearchResult(_)
                | ContentBlock::ServerToolUse(_)
                | ContentBlock::WebSearchToolResult(_)
                | ContentBlock::WebFetchToolResult(_)
                | ContentBlock::CodeExecutionToolResult(_)
                | ContentBlock::BashCodeExecutionToolResult(_)
                | ContentBlock::TextEditorCodeExecutionToolResult(_)
                | ContentBlock::ToolSearchToolResult(_)
                | ContentBlock::ContainerUpload(_) => {}
            }
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }

    /// Convert `ToolDefinition` to `ClaudeTool`.
    ///
    /// 将 `ToolDefinition` 转换为 `ClaudeTool`。
    pub fn tool_to_claude_tool(tool: &ToolDefinition) -> ClaudeTool {
        let input_schema = tool
            .function
            .parameters
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok())
            .unwrap_or(Value::Null);

        ClaudeTool::Custom(ClaudeCustomTool {
            name: tool.function.name.clone(),
            description: tool.function.description.clone(),
            input_schema,
            cache_control: None,
            r#type: Some("custom".into()),
            strict: tool.function.strict,
            allowed_callers: None,
        })
    }

    /// Convert `ToolChoice` to `ClaudeToolChoice`.
    ///
    /// 将 `ToolChoice` 转换为 `ClaudeToolChoice`。
    pub fn tool_choice_to_claude(choice: &ToolChoice) -> ClaudeToolChoice {
        match choice {
            ToolChoice::None => ClaudeToolChoice::None,
            ToolChoice::Auto => ClaudeToolChoice::Auto {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Required => ClaudeToolChoice::Any {
                disable_parallel_tool_use: None,
            },
            ToolChoice::Function { name } => ClaudeToolChoice::Tool {
                name: name.clone(),
                disable_parallel_tool_use: None,
            },
        }
    }

    /// Convert `ClaudeTool` to `ToolDefinition`.
    ///
    /// 将 `ClaudeTool` 转换为 `ToolDefinition`。
    pub fn tool_from_claude(value: ClaudeTool) -> Result<ToolDefinition, MapperError> {
        match value {
            ClaudeTool::Custom(custom) => Ok(ToolDefinition {
                r#type: ToolType::Function,
                function: FunctionDefinition {
                    name: custom.name,
                    description: custom.description,
                    parameters: Some(
                        serde_json::from_value(custom.input_schema)
                            .map_err(MapperError::JsonError)?,
                    ),
                    strict: custom.strict,
                },
            }),
            _ => Err(MapperError::UnsupportedContent {
                role: "Tool".to_string(),
                protocol: "Claude".to_string(),
            }),
        }
    }

    /// Convert `ClaudeToolChoice` to `ToolChoice`.
    ///
    /// 将 `ClaudeToolChoice` 转换为 `ToolChoice`。
    pub fn tool_choice_from_claude(value: ClaudeToolChoice) -> ToolChoice {
        match value {
            ClaudeToolChoice::None => ToolChoice::None,
            ClaudeToolChoice::Auto { .. } => ToolChoice::Auto,
            ClaudeToolChoice::Any { .. } => ToolChoice::Required,
            ClaudeToolChoice::Tool { name, .. } => ToolChoice::Function { name },
        }
    }
}

impl TryFrom<Message> for ClaudeMessage {
    type Error = MapperError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        ClaudeMessagesMapper::from_core_message(msg)
    }
}

impl TryFrom<MessagesResponse> for Message {
    type Error = MapperError;

    fn try_from(resp: MessagesResponse) -> Result<Self, Self::Error> {
        ClaudeMessagesMapper::to_core_message(resp)
    }
}

impl TryFrom<&ToolDefinition> for ClaudeTool {
    type Error = MapperError;

    fn try_from(tool: &ToolDefinition) -> Result<Self, Self::Error> {
        Ok(ClaudeMessagesMapper::tool_to_claude_tool(tool))
    }
}

impl TryFrom<&ToolChoice> for ClaudeToolChoice {
    type Error = MapperError;

    fn try_from(choice: &ToolChoice) -> Result<Self, Self::Error> {
        Ok(ClaudeMessagesMapper::tool_choice_to_claude(choice))
    }
}

impl TryFrom<ClaudeTool> for ToolDefinition {
    type Error = MapperError;

    fn try_from(value: ClaudeTool) -> Result<Self, Self::Error> {
        ClaudeMessagesMapper::tool_from_claude(value)
    }
}

impl TryFrom<ClaudeToolChoice> for ToolChoice {
    type Error = MapperError;

    fn try_from(value: ClaudeToolChoice) -> Result<Self, Self::Error> {
        Ok(ClaudeMessagesMapper::tool_choice_from_claude(value))
    }
}

/// Claude Messages Raw Conversation State.
///
/// Claude Messages 原始对话状态。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessagesConversationState {
    /// System Prompt.
    ///
    /// 系统提示词(可选)。
    pub system_prompt: Option<StaticRefStr>,

    /// Raw Message list.
    ///
    /// 原始消息列表。
    pub messages: Vec<ClaudeMessage>,

    /// Available raw tools.
    ///
    /// 可用原始工具列表。
    pub tools: Vec<ClaudeTool>,

    /// Raw Tool choice preference.
    ///
    /// 原始工具选择偏好。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ClaudeToolChoice>,
}

impl ConversationStateTrait for MessagesConversationState {}

impl TryFrom<ConversationState> for MessagesConversationState {
    type Error = MapperError;

    fn try_from(state: ConversationState) -> Result<Self, Self::Error> {
        let raw = state.into_raw();
        let messages = raw
            .messages
            .into_iter()
            .map(ClaudeMessagesMapper::from_core_message)
            .collect::<Result<Vec<_>, _>>()?;
        let tools = raw
            .tools
            .iter()
            .map(ClaudeMessagesMapper::tool_to_claude_tool)
            .collect();
        let tool_choice = raw
            .tool_choice
            .as_ref()
            .map(ClaudeMessagesMapper::tool_choice_to_claude);

        Ok(MessagesConversationState {
            system_prompt: raw.system_prompt,
            messages,
            tools,
            tool_choice,
        })
    }
}

/// A stateful mapper for Claude streaming responses.
///
/// 用于 Claude 流式响应的有状态映射器。
pub struct ClaudeMessagesStreamMapper;

impl ClaudeMessagesStreamMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map_response(&mut self, event: ClaudeStreamEvent) -> Result<DeltaMessage, MapperError> {
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
                    reasoning_tokens: message
                        .usage
                        .output_tokens_details
                        .and_then(|v| v.thinking_tokens),
                    cached_input_tokens: message.usage.cache_read_input_tokens,
                    cached_output_tokens: None,
                    tool_use_tokens: None,
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
                            r#type: Some("function".into()),
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
                            text: "".into(),
                            signature: Some(t.data), // Encrypted/Redacted data as signature
                        })
                    }
                    ChunkContentBlock::ServerToolUse(stu) => {
                        Some(DeltaContentPart::ToolCall(DeltaToolCall {
                            index,
                            id: Some(stu.id),
                            r#type: Some("server_function".into()),
                            function: Some(DeltaFunction {
                                name: Some(stu.name),
                                arguments: Some(stu.input.to_string().into()),
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
                            if let WebSearchResultItem::WebSearchResult(res) = item {
                                use std::fmt::Write;
                                let _ = writeln!(
                                    text,
                                    "- {} ({}) : {}",
                                    res.title, res.url, res.encrypted_content
                                );
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
                    ChunkContentBlock::Image(_)
                    | ChunkContentBlock::WebFetchToolResult(_)
                    | ChunkContentBlock::CodeExecutionToolResult(_)
                    | ChunkContentBlock::BashCodeExecutionToolResult(_)
                    | ChunkContentBlock::TextEditorCodeExecutionToolResult(_)
                    | ChunkContentBlock::ToolSearchToolResult(_)
                    | ChunkContentBlock::ContainerUpload(_) => {
                        // Other streaming block types do not produce user text deltas.
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
                    ChunkContentBlockDelta::TextDelta { text } => Some(DeltaContentPart::Text {
                        index,
                        text,
                        signature: None,
                    }),
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
                            text: "".into(),
                            signature: Some(signature),
                        })
                    }
                    ChunkContentBlockDelta::CitationsDelta { .. } => None,
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
                    StopReason::PauseTurn => crate::message::FinishReason::PauseTurn,
                    StopReason::Refusal => crate::message::FinishReason::Refusal,
                });

                let input_tokens = usage.input_tokens.unwrap_or(0);
                let usage = crate::message::Usage {
                    input_tokens,
                    output_tokens: usage.output_tokens,
                    total_tokens: input_tokens + usage.output_tokens,
                    ..Default::default()
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

impl Default for ClaudeMessagesStreamMapper {
    fn default() -> Self {
        Self::new()
    }
}
