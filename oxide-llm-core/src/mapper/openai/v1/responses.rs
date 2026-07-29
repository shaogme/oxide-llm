use crate::mapper::MapperError;
use crate::message::{
    ContentPart, DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, FinishReason,
    ImageSource, Message, Role, Usage,
};
use crate::tool::ToolCall;
use oxide_llm_proto::openai::v1::response::chunk::ResponseStreamEvent;
use oxide_llm_proto::openai::v1::response::response::{OutputItem, ResponseStatus};

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
    ) -> Result<Vec<oxide_llm_proto::openai::v1::response::request::InputItem>, MapperError> {
        use oxide_llm_proto::openai::v1::response::request::{
            InputAudioContent, InputContentPart, InputItem, InputMessage, InputMessageContent,
        };

        let role_str = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };

        let mut items = Vec::new();

        if msg.role == Role::Tool {
            for part in msg.content {
                match part {
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

                        items.push(InputItem::FunctionCallOutput {
                            call_id: res.tool_call_id,
                            output: content_str,
                            id: None,
                        });
                    }
                    _ => return Err(MapperError::MissingToolResult),
                }
            }
            return Ok(items);
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
                    if tc.name.is_empty() {
                        return Err(MapperError::MissingField {
                            field: "tool_call.function.name".to_string(),
                        });
                    }
                    let arguments = match &tc.arguments {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    items.push(InputItem::FunctionCall {
                        call_id: tc.id,
                        name: tc.name,
                        arguments,
                        id: None,
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
                _ => {}
            }
        }

        if !parts.is_empty() {
            let content = if parts.len() == 1
                && let InputContentPart::InputText { text } = &parts[0]
            {
                InputMessageContent::String(text.clone())
            } else {
                InputMessageContent::Parts(parts)
            };

            items.insert(
                0,
                InputItem::Message(InputMessage {
                    role: role_str.into(),
                    content,
                    name: msg.name,
                    status: None,
                }),
            );
        }

        Ok(items)
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
                    if fc.name.is_empty() {
                        return Err(MapperError::MissingField {
                            field: "tool_call.function.name".to_string(),
                        });
                    }
                    let arguments =
                        serde_json::from_str(&fc.arguments).map_err(MapperError::JsonError)?;
                    content_parts.push(ContentPart::ToolCall(ToolCall {
                        id: fc.call_id.or(fc.id).unwrap_or_default().into(),
                        name: fc.name.into(),
                        arguments,
                        signature: None,
                    }));
                }
                OutputItem::Reasoning(reasoning) => {
                    if let Some(summary_vals) = reasoning.summary {
                        let text: String = summary_vals
                            .iter()
                            .filter_map(|v| v.text.as_deref())
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

/// Stateful mapper for OpenAI Response API streaming events.
///
/// 用于 OpenAI Response API 流式事件的有状态映射器。
pub struct OpenAIResponseStreamMapper;

impl OpenAIResponseStreamMapper {
    /// Create a new `OpenAIResponseStreamMapper`.
    ///
    /// 创建一个新的 `OpenAIResponseStreamMapper`。
    pub fn new() -> Self {
        Self
    }

    /// Map OpenAI Response API stream event to a core DeltaMessage.
    ///
    /// 将 OpenAI Response API 流式事件映射为核心 DeltaMessage。
    pub fn map_response(
        &mut self,
        event: ResponseStreamEvent,
    ) -> Result<DeltaMessage, MapperError> {
        match event {
            ResponseStreamEvent::OutputTextDelta {
                content_index,
                delta,
                ..
            } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::Text {
                    index: content_index,
                    text: delta,
                    signature: None,
                }]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::RefusalDelta { delta, .. } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::Refusal {
                    refusal: delta.into(),
                }]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::ReasoningTextDelta {
                content_index,
                delta,
                ..
            } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::Reasoning {
                    index: content_index,
                    text: delta,
                    signature: None,
                }]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::FunctionCallArgumentsDelta {
                output_index,
                call_id,
                delta,
                ..
            } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                    index: output_index,
                    id: call_id,
                    r#type: Some("function".into()),
                    function: Some(DeltaFunction {
                        name: None,
                        arguments: Some(delta.into()),
                    }),
                    signature: None,
                })]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::OutputItemAdded {
                output_index,
                item: OutputItem::FunctionCall(fc),
                ..
            } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                    index: output_index,
                    id: fc.call_id.or(fc.id).map(Into::into),
                    r#type: Some("function".into()),
                    function: Some(DeltaFunction {
                        name: Some(fc.name.into()),
                        arguments: if fc.arguments.is_empty() {
                            None
                        } else {
                            Some(fc.arguments.into())
                        },
                    }),
                    signature: None,
                })]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::OutputItemAdded { .. } => Ok(DeltaMessage::default()),
            ResponseStreamEvent::AudioTranscriptDelta {
                content_index,
                delta,
                ..
            } => Ok(DeltaMessage {
                role: None,
                content: Some(vec![DeltaContentPart::Text {
                    index: content_index,
                    text: delta,
                    signature: None,
                }]),
                name: None,
                finish_reason: None,
                usage: None,
            }),
            ResponseStreamEvent::Completed { response, .. } => {
                let usage = response.usage.map(|u| Usage {
                    input_tokens: u.input_tokens,
                    output_tokens: u.output_tokens,
                    total_tokens: u.total_tokens,
                });
                let has_function_call = response
                    .output
                    .iter()
                    .any(|item| matches!(item, OutputItem::FunctionCall(_)));
                let finish_reason = if has_function_call {
                    Some(FinishReason::ToolCalls)
                } else {
                    match response.status {
                        ResponseStatus::Completed => Some(FinishReason::Stop),
                        ResponseStatus::Incomplete => Some(FinishReason::Length),
                        ResponseStatus::Failed => Some(FinishReason::Other("failed".into())),
                        ResponseStatus::Cancelled => Some(FinishReason::Other("cancelled".into())),
                        _ => None,
                    }
                };
                Ok(DeltaMessage {
                    role: None,
                    content: None,
                    name: None,
                    finish_reason,
                    usage,
                })
            }
            ResponseStreamEvent::Created { .. } => Ok(DeltaMessage {
                role: Some(Role::Assistant),
                content: None,
                name: None,
                finish_reason: None,
                usage: None,
            }),
            _ => Ok(DeltaMessage::default()),
        }
    }
}

impl Default for OpenAIResponseStreamMapper {
    fn default() -> Self {
        Self::new()
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
                id: Some("msg_123".into()),
                role: Some("assistant".into()),
                content: vec![OutputMessageContent::OutputText(OutputTextContent {
                    text: "Hello world".to_string(),
                    annotations: None,
                    logprobs: None,
                })],
                status: Some(ResponseStatus::Completed),
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

    #[test]
    fn test_openai_response_stream_mapper() {
        let mut mapper = OpenAIResponseStreamMapper::new();
        let event = ResponseStreamEvent::OutputTextDelta {
            item_id: "msg_123".into(),
            output_index: 0,
            content_index: 0,
            delta: "Hello".to_string(),
            logprobs: None,
            sequence_number: Some(1),
        };
        let delta = mapper.map_response(event).unwrap();
        assert!(delta.content.is_some());
        let parts = delta.content.unwrap();
        assert_eq!(parts.len(), 1);
        if let DeltaContentPart::Text { text, .. } = &parts[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text delta part");
        }
    }

    #[test]
    fn test_openai_response_mapper_empty_tool_name_error() {
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

        let err = OpenAIResponseMapper::from_core_message(msg).unwrap_err();
        assert!(matches!(err, MapperError::MissingField { ref field } if field == "tool_call.function.name"));
    }

    #[test]
    fn test_openai_response_mapper_tool_call_and_result_items() {
        use oxide_llm_proto::openai::v1::response::request::InputItem;

        let assistant_msg = Message {
            role: Role::Assistant,
            content: vec![ContentPart::ToolCall(ToolCall {
                id: "call_101".into(),
                name: "get_weather".into(),
                arguments: serde_json::json!({"location":"Tokyo"}),
                signature: None,
            })],
            name: None,
        };

        let items = OpenAIResponseMapper::from_core_message(assistant_msg).unwrap();
        assert_eq!(items.len(), 1);
        if let InputItem::FunctionCall {
            call_id,
            name,
            arguments,
            id: _,
        } = &items[0]
        {
            assert_eq!(call_id, "call_101");
            assert_eq!(name, "get_weather");
            assert_eq!(arguments, r#"{"location":"Tokyo"}"#);
        } else {
            panic!("Expected InputItem::FunctionCall");
        }

        let tool_msg = Message {
            role: Role::Tool,
            content: vec![crate::message::ContentPart::ToolResult(
                crate::tool::ToolResult {
                    tool_call_id: "call_101".into(),
                    name: "get_weather".into(),
                    content: vec![ContentPart::Text {
                        text: "Sunny".into(),
                        signature: None,
                    }],
                    is_error: false,
                    signature: None,
                },
            )],
            name: None,
        };

        let result_items = OpenAIResponseMapper::from_core_message(tool_msg).unwrap();
        assert_eq!(result_items.len(), 1);
        if let InputItem::FunctionCallOutput {
            call_id,
            output,
            id: _,
        } = &result_items[0] {
            assert_eq!(call_id, "call_101");
            assert_eq!(output, "Sunny");
        } else {
            panic!("Expected InputItem::FunctionCallOutput");
        }
    }
}
