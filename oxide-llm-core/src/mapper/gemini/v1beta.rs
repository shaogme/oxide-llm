use crate::mapper::MapperError;
use crate::message::{ContentPart, ImageSource, Message, Role};
use crate::tool::ToolCall;

use crate::message::{DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall};
use oxide_llm_proto::gemini::v1beta::generate_content::response::GenerateContentResponse;
use oxide_llm_proto::gemini::v1beta::generate_content::{
    Blob, Content as GeminiContent, FileData, FunctionCall, FunctionResponse, Part as GeminiPart,
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
            Role::Tool => ("user", convert_content_to_gemini_parts(msg.content)?),
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
                gemini_parts.push(GeminiPart {
                    text: Some(text),
                    ..Default::default()
                });
            }
            ContentPart::Image(image) => {
                // Gemini usually expects InlineData for base64
                if let ImageSource::Base64 { data } = image.source {
                    gemini_parts.push(GeminiPart {
                        inline_data: Some(Blob {
                            mime_type: image.media_type.ok_or(MapperError::InvalidMediaType)?,
                            data,
                        }),
                        ..Default::default()
                    });
                } else {
                    if let ImageSource::Url { url } = image.source {
                        gemini_parts.push(GeminiPart {
                            file_data: Some(FileData {
                                mime_type: image.media_type,
                                file_uri: url,
                            }),
                            ..Default::default()
                        });
                    }
                }
            }
            ContentPart::Audio(audio) => {
                gemini_parts.push(GeminiPart {
                    inline_data: Some(Blob {
                        mime_type: format!("audio/{}", audio.format), // e.g. audio/mp3
                        data: audio.data,
                    }),
                    ..Default::default()
                });
            }
            ContentPart::ToolCall(tc) => {
                gemini_parts.push(GeminiPart {
                    function_call: Some(FunctionCall {
                        id: Some(tc.id),
                        name: tc.name,
                        args: tc.arguments,
                    }),
                    ..Default::default()
                });
            }
            ContentPart::ToolResult(tr) => {
                let response_value = if tr.content.len() == 1 {
                    match &tr.content[0] {
                        ContentPart::Text { text } => serde_json::json!({ "content": text }),
                        ContentPart::Json(value) => value.clone(),
                        _ => {
                            // Try to serialize other parts
                            serde_json::to_value(&tr.content)?
                        }
                    }
                } else {
                    serde_json::to_value(&tr.content)?
                };

                gemini_parts.push(GeminiPart {
                    function_response: Some(FunctionResponse {
                        id: Some(tr.tool_call_id),
                        name: tr.name,
                        response: response_value,
                        parts: None,
                        will_continue: None,
                        scheduling: None,
                    }),
                    ..Default::default()
                });
            }
            ContentPart::Json(value) => {
                let text = serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                gemini_parts.push(GeminiPart {
                    text: Some(text),
                    ..Default::default()
                });
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
            if let Some(text) = &part.text {
                content_parts.push(ContentPart::Text { text: text.clone() });
            } else if let Some(fc) = &part.function_call {
                // Interoperability: Gemini doesn't have call_id, so use name as ID.
                content_parts.push(ContentPart::ToolCall(ToolCall {
                    id: fc.id.clone().unwrap_or_else(|| fc.name.clone()),
                    name: fc.name.clone(),
                    arguments: fc.args.clone(),
                }));
            }
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }
}

/// Convert Gemini GenerateContentResponse to core DeltaMessage.
///
/// 将 Gemini GenerateContentResponse 转换为核心 DeltaMessage。
impl TryFrom<GenerateContentResponse> for DeltaMessage {
    type Error = MapperError;

    fn try_from(resp: GenerateContentResponse) -> Result<Self, Self::Error> {
        let candidate = resp.candidates.first();

        // Usage info might be available at top level
        let usage = resp.usage_metadata.map(|u| crate::message::Usage {
            input_tokens: u.prompt_token_count as u32,
            output_tokens: u.candidates_token_count as u32,
            total_tokens: u.total_token_count as u32,
        });

        if candidate.is_none() {
            // Just usage update or empty
            return Ok(DeltaMessage {
                role: None,
                content: None,
                name: None,
                finish_reason: None,
                usage,
            });
        }
        let candidate = candidate.unwrap();

        let finish_reason = candidate.finish_reason.as_ref().map(|r| match r {
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Stop => {
                crate::message::FinishReason::Stop
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::MaxTokens => {
                crate::message::FinishReason::Length
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Safety => {
                crate::message::FinishReason::ContentFilter
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Recitation => {
                crate::message::FinishReason::ContentFilter
            }
            oxide_llm_proto::gemini::v1beta::generate_content::response::FinishReason::Other => {
                crate::message::FinishReason::Other("Other".to_string())
            }
            // Gemini specific ones
            _ => crate::message::FinishReason::Other(format!("{:?}", r)),
        });

        let mut content_parts = Vec::new();

        for (i, part) in candidate.content.parts.iter().enumerate() {
            if part.thought == Some(true) {
                if let Some(text) = &part.text {
                    content_parts.push(DeltaContentPart::Reasoning {
                        index: i as u32,
                        text: text.clone(),
                        signature: part.thought_signature.clone(),
                    });
                }
            } else if let Some(text) = &part.text {
                content_parts.push(DeltaContentPart::Text {
                    index: i as u32,
                    text: text.clone(),
                });
            } else if let Some(fc) = &part.function_call {
                // Gemini sends full function call in one go usually, but for Delta we map it
                content_parts.push(DeltaContentPart::ToolCall(DeltaToolCall {
                    index: i as u32,
                    id: Some(fc.id.clone().unwrap_or_else(|| fc.name.clone())),
                    r#type: Some("function".to_string()),
                    function: Some(DeltaFunction {
                        name: Some(fc.name.clone()),
                        arguments: Some(
                            serde_json::to_string(&fc.args).map_err(MapperError::JsonError)?,
                        ),
                    }),
                }));
            }
        }

        let role = candidate.content.role.as_deref().map(|r| match r {
            "user" => Role::User,
            "model" => Role::Assistant,
            "function" => Role::Tool,
            _ => Role::Assistant,
        });

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
