use crate::mapper::MapperError;
use crate::message::{ContentPart, ImageSource, Message, Role};
use crate::tool::ToolCall;

/// Mapper for OpenAI Response API protocol.
///
/// OpenAI Response 协议映射器。
pub struct OpenAIResponseMapper;

impl OpenAIResponseMapper {
    /// Convert core Message to OpenAI Response InputItem.
    ///
    /// 将核心 Message 转换为 OpenAI Response InputItem。
    pub fn from_core_message(
        msg: Message,
    ) -> Result<oxide_llm_proto::openai::v1::response::request::InputItem, MapperError> {
        use oxide_llm_proto::openai::v1::response::request::{
            InputAudioContent, InputContentPart, InputItem, InputMessage, InputMessageContent,
        };

        let role_str = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };

        if msg.content.len() == 1
            && let ContentPart::Text { text, signature: _ } = &msg.content[0]
        {
            return Ok(InputItem::Message(InputMessage {
                role: role_str.into(),
                content: InputMessageContent::String(text.clone()),
                name: msg.name,
                status: None,
            }));
        }

        let mut parts = Vec::new();
        for part in msg.content {
            match part {
                ContentPart::Text { text, signature: _ } => {
                    parts.push(InputContentPart::InputText { text });
                }
                ContentPart::Image(image) => {
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
                    parts.push(InputContentPart::InputImage { image_url: url });
                }
                ContentPart::Audio(audio) => {
                    parts.push(InputContentPart::InputAudio {
                        input_audio: InputAudioContent {
                            data: audio.data,
                            format: audio.format,
                        },
                    });
                }
                ContentPart::Json(value) => {
                    let text = serde_json::to_string(&value).map_err(MapperError::JsonError)?;
                    parts.push(InputContentPart::InputText { text });
                }
                ContentPart::ToolCall(tc) => {
                    let text = serde_json::to_string(&tc).map_err(MapperError::JsonError)?;
                    parts.push(InputContentPart::InputText { text });
                }
                ContentPart::ToolResult(res) => {
                    let content_str = if res.content.len() == 1 {
                        match &res.content[0] {
                            ContentPart::Text { text, signature: _ } => text.clone(),
                            ContentPart::Json(value) => {
                                serde_json::to_string(value).map_err(MapperError::JsonError)?
                            }
                            _ => serde_json::to_string(&res.content)?,
                        }
                    } else {
                        serde_json::to_string(&res.content)?
                    };
                    parts.push(InputContentPart::InputText {
                        text: content_str.into(),
                    });
                }
                ContentPart::Refusal { refusal } => {
                    parts.push(InputContentPart::InputText {
                        text: refusal.to_string(),
                    });
                }
                ContentPart::Reasoning { text, signature: _ } => {
                    parts.push(InputContentPart::InputText { text });
                }
            }
        }

        Ok(InputItem::Message(InputMessage {
            role: role_str.into(),
            content: InputMessageContent::Parts(parts),
            name: msg.name,
            status: None,
        }))
    }

    /// Convert OpenAI Response object to core Message.
    ///
    /// 将 OpenAI Response 转换为核心 Message。
    pub fn to_core_message(
        resp: oxide_llm_proto::openai::v1::response::response::Response,
    ) -> Result<Message, MapperError> {
        use oxide_llm_proto::openai::v1::response::response::{OutputItem, OutputMessageContent};

        let mut content_parts = Vec::new();

        for item in resp.output {
            match item {
                OutputItem::Message(msg) => {
                    for content in msg.content {
                        match content {
                            OutputMessageContent::OutputText(text_content) => {
                                content_parts.push(ContentPart::Text {
                                    text: text_content.text,
                                    signature: None,
                                });
                            }
                            OutputMessageContent::Refusal(refusal_content) => {
                                content_parts.push(ContentPart::Refusal {
                                    refusal: refusal_content.refusal,
                                });
                            }
                        }
                    }
                }
                OutputItem::FunctionCall(fc) => {
                    let arguments =
                        serde_json::from_str(&fc.arguments).map_err(MapperError::JsonError)?;
                    content_parts.push(ContentPart::ToolCall(ToolCall {
                        id: fc.id.into(),
                        name: fc.name.into(),
                        arguments,
                        signature: None,
                    }));
                }
                OutputItem::Reasoning(reasoning) => {
                    if let Some(summary_vals) = reasoning.summary {
                        let text: String = summary_vals
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !text.is_empty() {
                            content_parts.push(ContentPart::Reasoning {
                                text,
                                signature: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        if content_parts.is_empty() {
            return Err(MapperError::EmptyResponse);
        }

        Ok(Message {
            role: Role::Assistant,
            content: content_parts,
            name: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_llm_proto::openai::v1::response::response::{
        OutputItem, OutputMessage, OutputMessageContent, OutputTextContent, Response,
        ResponseStatus,
    };

    #[test]
    fn test_openai_response_mapper_to_core_message() {
        let resp = Response {
            id: "resp_123".into(),
            object: "response".into(),
            status: ResponseStatus::Completed,
            created_at: 1741476542,
            completed_at: Some(1741476543),
            error: None,
            incomplete_details: None,
            output: vec![OutputItem::Message(OutputMessage {
                id: "msg_123".into(),
                role: "assistant".into(),
                content: vec![OutputMessageContent::OutputText(OutputTextContent {
                    text: "Hello world".to_string(),
                    annotations: None,
                    logprobs: None,
                })],
                status: ResponseStatus::Completed,
            })],
            instructions: None,
            metadata: None,
            model: Some("gpt-4.1-2025-04-14".into()),
            top_logprobs: None,
            temperature: Some(1.0),
            top_p: Some(1.0),
            user: None,
            service_tier: None,
            system_fingerprint: None,
            usage: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: Some(true),
            conversation: None,
            previous_response_id: None,
            reasoning: None,
            background: None,
            max_output_tokens: None,
            max_tool_calls: None,
            text: None,
            prompt: None,
            truncation: None,
        };

        let core_msg = OpenAIResponseMapper::to_core_message(resp).unwrap();
        assert_eq!(core_msg.role, Role::Assistant);
        assert_eq!(core_msg.content.len(), 1);
        if let ContentPart::Text { text, .. } = &core_msg.content[0] {
            assert_eq!(text, "Hello world");
        } else {
            panic!("Expected text content part");
        }
    }
}
