use crate::message::{Audio, ContentPart, Image, ImageSource, Message, Role};
use crate::tool::ToolCall;

use error_set::error_set;

error_set! {
    MapperError := {
        #[display("Unsupported content part for role {role} in {protocol}")]
        UnsupportedContent {
            role: String,
            protocol: String
        },
        #[display("Missing required field: {field}")]
        MissingField {
            field: String
        },
        #[display("JSON serialization error: {0}")]
        JsonError(serde_json::Error),
        #[display("Invalid media type")]
        InvalidMediaType,
        #[display("OpenAI Tool messages must correspond to exactly one ToolResult")]
        InvalidOpenAIToolMessage,
        #[display("Message with Tool role must contain ToolResult")]
        MissingToolResult,
        #[display("No choices/candidates found in response")]
        EmptyResponse,
    }
}

// =================================================================================================
// OpenAI Mapper
// =================================================================================================
pub mod openai {
    use super::*;
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
}

// =================================================================================================
// Claude Mapper
// =================================================================================================
pub mod claude {
    use super::*;
    use oxide_llm_proto::claude::v1::messages::request::Message as ClaudeMessage;
    use oxide_llm_proto::claude::v1::messages::response::MessagesResponse;
    use oxide_llm_proto::claude::v1::messages::{
        Content as ClaudeContent, ContentBlock, ImageBlock, ImageSource as ClaudeImageSource,
        Role as ClaudeRole, TextBlock, ToolResultBlock, ToolResultContent, ToolUseBlock,
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
                ContentPart::Text { text } => {
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
                            ContentPart::Text { text } => ToolResultContent::Text(text.clone()),
                            _ => ToolResultContent::Blocks(convert_content_to_claude_blocks(
                                tr.content,
                            )?),
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
                        });
                    }
                    ContentBlock::ToolUse(tool_use) => {
                        content_parts.push(ContentPart::ToolCall(ToolCall {
                            id: tool_use.id,
                            name: tool_use.name,
                            arguments: tool_use.input,
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
}

// =================================================================================================
// Gemini Mapper
// =================================================================================================
pub mod gemini {
    use super::*;
    use oxide_llm_proto::gemini::v1beta::response::GenerateContentResponse;
    use oxide_llm_proto::gemini::v1beta::{
        Blob, Content as GeminiContent, FileData, FunctionCall, FunctionResponse,
        Part as GeminiPart,
    };

    /// Convert core Message to Gemini Content.
    ///
    /// 将核心 Message 转换为 Gemini Content。
    impl TryFrom<Message> for GeminiContent {
        type Error = MapperError;

        fn try_from(msg: Message) -> Result<Self, Self::Error> {
            let (role, parts) = match msg.role {
                Role::User => ("user", convert_content_to_gemini_parts(msg.content)?),
                Role::Assistant => ("model", convert_content_to_gemini_parts(msg.content)?),
                Role::Tool => ("function", convert_content_to_gemini_parts(msg.content)?),
            };

            Ok(GeminiContent {
                parts,
                role: Some(role.to_string()),
            })
        }
    }

    fn convert_content_to_gemini_parts(
        parts: Vec<ContentPart>,
    ) -> Result<Vec<GeminiPart>, MapperError> {
        let mut gemini_parts = Vec::new();
        for part in parts {
            match part {
                ContentPart::Text { text } => {
                    gemini_parts.push(GeminiPart::Text(text));
                }
                ContentPart::Image(image) => {
                    // Gemini usually expects InlineData for base64
                    if let ImageSource::Base64 { data } = image.source {
                        gemini_parts.push(GeminiPart::InlineData(Blob {
                            mime_type: image.media_type.ok_or(MapperError::InvalidMediaType)?,
                            data,
                        }));
                    } else {
                        // Handle URL images if strictly needed (FileData), but requires upload first usually.
                        // Here mapping standard Image to generic structure might be hard if we don't know if it's uploaded.
                        // Assuming FileData for URL might be valid if the URL is a gs:// URI or similar,
                        // but standard HTTP URL might not work directly.
                        // For now, let's treat it as FileData and hope the URL is valid for Gemini.
                        if let ImageSource::Url { url } = image.source {
                            gemini_parts.push(GeminiPart::FileData(FileData {
                                mime_type: image.media_type,
                                file_uri: url,
                            }));
                        }
                    }
                }
                ContentPart::Audio(audio) => {
                    gemini_parts.push(GeminiPart::InlineData(Blob {
                        mime_type: format!("audio/{}", audio.format), // e.g. audio/mp3
                        data: audio.data,
                    }));
                }
                ContentPart::ToolCall(tc) => {
                    gemini_parts.push(GeminiPart::FunctionCall(FunctionCall {
                        name: tc.name,
                        args: tc.arguments,
                    }));
                }
                ContentPart::ToolResult(tr) => {
                    // Gemini FunctionResponse contains a JSON object.
                    // We need to convert our content to a JSON Value.
                    // If content is just text, wrap it.
                    // If content is structured, use it.
                    // NOTE: Gemini v1beta FunctionResponse expects `response: Value`.

                    let response_value = if tr.content.len() == 1 {
                        match &tr.content[0] {
                            ContentPart::Text { text } => serde_json::json!({ "content": text }),
                            // If it's already a JSON-like structure (e.g. from a tool output that was parsed),
                            // we might need a way to pass raw JSON.
                            // But `ContentPart` doesn't have a RawJson variant.
                            // Assuming generic Text for now.
                            _ => {
                                // Try to serialize other parts
                                serde_json::to_value(&tr.content)?
                            }
                        }
                    } else {
                        serde_json::to_value(&tr.content)?
                    };

                    gemini_parts.push(GeminiPart::FunctionResponse(FunctionResponse {
                        name: tr.name,
                        response: response_value,
                    }));
                }
                _ => {
                    return Err(MapperError::UnsupportedContent {
                        role: "Any".to_string(),
                        protocol: "Gemini".to_string(),
                    });
                }
            }
        }
        Ok(gemini_parts)
    }

    /// Convert Gemini GenerateContentResponse to core Message.
    ///
    /// 将 Gemini GenerateContentResponse 转换为核心 Message。
    impl TryFrom<GenerateContentResponse> for Message {
        type Error = MapperError;

        fn try_from(resp: GenerateContentResponse) -> Result<Self, Self::Error> {
            let candidate = resp.candidates.first().ok_or(MapperError::EmptyResponse)?;

            let mut content_parts = Vec::new();

            for part in &candidate.content.parts {
                match part {
                    GeminiPart::Text(text) => {
                        content_parts.push(ContentPart::Text { text: text.clone() });
                    }
                    GeminiPart::FunctionCall(fc) => {
                        // Interoperability: Gemini doesn't have call_id, so use name as ID.
                        content_parts.push(ContentPart::ToolCall(ToolCall {
                            id: fc.name.clone(),
                            name: fc.name.clone(),
                            arguments: fc.args.clone(),
                        }));
                    }
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
}
