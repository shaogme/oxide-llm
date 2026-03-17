use crate::mapper::MapperError;
use crate::message::{ContentPart, ImageSource, Message, Role};
use crate::tool::ToolCall;

use crate::message::{DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall};
use oxide_llm_proto::gemini::v1beta::generate_content::response::GenerateContentResponse;
use oxide_llm_proto::gemini::v1beta::generate_content::{
    Blob, Content as GeminiContent, FileData, FunctionCall, FunctionResponse, Part as GeminiPart,
};

/// Mapper for Gemini protocol.
///
/// Gemini 协议映射器。
pub struct GeminiMapper;

impl GeminiMapper {
    /// Convert core Message to Gemini Content.
    ///
    /// 将核心 Message 转换为 Gemini Content。
    pub fn from_core_message(msg: Message) -> Result<GeminiContent, MapperError> {
        let (role, parts) = match msg.role {
            Role::User => ("user", Self::convert_content_to_gemini_parts(msg.content)?),
            Role::Assistant => ("model", Self::convert_content_to_gemini_parts(msg.content)?),
            Role::Tool => ("user", Self::convert_content_to_gemini_parts(msg.content)?),
        };

        Ok(GeminiContent {
            parts,
            role: Some(role.to_string()),
        })
    }

    fn convert_content_to_gemini_parts(
        parts: Vec<ContentPart>,
    ) -> Result<Vec<GeminiPart>, MapperError> {
        let mut gemini_parts = Vec::new();
        let mut last_signature = None;

        for part in parts {
            match part {
                ContentPart::Reasoning { text, signature } => {
                    if signature.is_some() {
                        last_signature = signature.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        text: Some(text),
                        thought: Some(true),
                        thought_signature: signature,
                        ..Default::default()
                    });
                }
                ContentPart::Text { text, signature } => {
                    let sig = signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        text: Some(text),
                        thought_signature: sig,
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
                    let sig = tc.signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        function_call: Some(FunctionCall {
                            id: Some(tc.id),
                            name: tc.name,
                            args: tc.arguments,
                        }),
                        thought_signature: sig,
                        ..Default::default()
                    });
                }
                ContentPart::ToolResult(tr) => {
                    let response_value = if tr.content.len() == 1 {
                        match &tr.content[0] {
                            ContentPart::Text { text, signature: _ } => {
                                serde_json::json!({ "content": text })
                            }
                            ContentPart::Json(value) => value.clone(),
                            _ => {
                                // Try to serialize other parts
                                serde_json::to_value(&tr.content)?
                            }
                        }
                    } else {
                        serde_json::to_value(&tr.content)?
                    };

                    let sig = tr.signature.or_else(|| last_signature.clone());
                    if sig.is_some() {
                        last_signature = sig.clone();
                    }
                    gemini_parts.push(GeminiPart {
                        function_response: Some(FunctionResponse {
                            id: Some(tr.tool_call_id),
                            name: tr.name,
                            response: response_value,
                            parts: None,
                            will_continue: None,
                            scheduling: None,
                        }),
                        thought_signature: sig,
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
    pub fn to_core_message(resp: GenerateContentResponse) -> Result<Message, MapperError> {
        let candidate = resp.candidates.first().ok_or(MapperError::EmptyResponse)?;

        let mut content_parts = Vec::new();

        let mut last_sig = None;
        for part in &candidate.content.parts {
            let sig = part.thought_signature.clone().or_else(|| last_sig.clone());
            if sig.is_some() {
                last_sig = sig.clone();
            }

            if let Some(text) = &part.text {
                if part.thought == Some(true) {
                    content_parts.push(ContentPart::Reasoning {
                        text: text.clone(),
                        signature: sig,
                    });
                } else {
                    content_parts.push(ContentPart::Text {
                        text: text.clone(),
                        signature: sig,
                    });
                }
            } else if let Some(fc) = &part.function_call {
                // Interoperability: Gemini doesn't have call_id, so use name as ID.
                content_parts.push(ContentPart::ToolCall(ToolCall {
                    id: fc.id.clone().unwrap_or_else(|| fc.name.clone()),
                    name: fc.name.clone(),
                    arguments: fc.args.clone(),
                    signature: sig,
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

/// A stateful mapper for Gemini streaming responses.
///
/// 用于 Gemini 流式响应的有状态映射器。
pub struct GeminiStreamMapper {
    last_signature: Option<String>,
}

impl Default for GeminiStreamMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiStreamMapper {
    pub fn new() -> Self {
        Self {
            last_signature: None,
        }
    }

    pub fn map_response(
        &mut self,
        resp: GenerateContentResponse,
    ) -> Result<DeltaMessage, MapperError> {
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
            _ => crate::message::FinishReason::Other(format!("{:?}", r)),
        });

        let mut content_parts = Vec::new();

        for (i, part) in candidate.content.parts.iter().enumerate() {
            // Update sticky signature
            if let Some(sig) = &part.thought_signature {
                self.last_signature = Some(sig.clone());
            }
            let sig = self.last_signature.clone();

            if part.thought == Some(true) {
                if let Some(text) = &part.text {
                    content_parts.push(DeltaContentPart::Reasoning {
                        index: i as u32,
                        text: text.clone(),
                        signature: sig,
                    });
                }
            } else if let Some(text) = &part.text {
                content_parts.push(DeltaContentPart::Text {
                    index: i as u32,
                    text: text.clone(),
                    signature: sig,
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
                    signature: sig,
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
