pub mod assembler;
pub mod delta;
pub mod model;
pub mod stream;

pub use assembler::MessageAssembler;
pub use delta::{
    DeltaContentPart, DeltaFunction, DeltaMessage, DeltaToolCall, FinishReason, Usage,
};
pub use model::{Audio, ContentPart, Image, ImageSource, Message, MessageHistory, Role};
pub use stream::{ChatStream, ChatStreamEvent};

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{executor::block_on, stream, StreamExt};

    #[test]
    fn test_message_and_history_builder() {
        let msg1 = Message::user("hello");
        assert_eq!(msg1.role, Role::User);
        assert_eq!(
            msg1.content,
            vec![ContentPart::Text {
                text: "hello".into(),
                signature: None,
            }]
        );

        let msg2 = Message::assistant("world");
        let history = MessageHistory::new()
            .with_message(msg1)
            .with_message(msg2);
        assert_eq!(history.messages.len(), 2);
    }

    #[test]
    fn test_message_assembler_interleaved_tool_calls() {
        let mut assembler = MessageAssembler::new();

        assembler.add(DeltaMessage {
            role: Some(Role::Assistant),
            content: Some(vec![
                DeltaContentPart::Text {
                    index: 0,
                    text: "Let me search for that.".into(),
                    signature: None,
                },
                DeltaContentPart::ToolCall(DeltaToolCall {
                    index: 1,
                    id: Some("call_abc123".into()),
                    r#type: Some("function".into()),
                    function: Some(DeltaFunction {
                        name: Some("search".into()),
                        arguments: Some("{\"query\":".into()),
                    }),
                    signature: None,
                }),
            ]),
            ..Default::default()
        });

        // Fragment without tool ID, relying on index -> synthetic/cached tool ID lookup
        assembler.add(DeltaMessage {
            content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                index: 1,
                id: None,
                r#type: None,
                function: Some(DeltaFunction {
                    name: None,
                    arguments: Some("\"rust\"}".into()),
                }),
                signature: None,
            })]),
            finish_reason: Some(FinishReason::ToolCalls),
            usage: Some(Usage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            }),
            ..Default::default()
        });

        assert_eq!(assembler.finish_reason(), Some(FinishReason::ToolCalls));
        assert_eq!(
            assembler.usage(),
            Some(Usage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            })
        );

        let assembled_tool = assembler.get_tool_call(1).unwrap();
        assert_eq!(assembled_tool.id, "call_abc123");
        assert_eq!(assembled_tool.name, "search");
        assert_eq!(
            assembled_tool.arguments,
            serde_json::json!({ "query": "rust" })
        );

        let final_msg = assembler.build();
        assert_eq!(final_msg.role, Role::Assistant);
        assert_eq!(final_msg.content.len(), 2);
        assert_eq!(
            final_msg.content[0],
            ContentPart::Text {
                text: "Let me search for that.".into(),
                signature: None,
            }
        );
        assert_eq!(final_msg.content[1], ContentPart::ToolCall(assembled_tool));
    }

    #[test]
    fn test_chat_stream_tool_call_events() {
        block_on(async {
            let deltas = vec![
                Ok::<_, String>(DeltaMessage {
                    role: Some(Role::Assistant),
                    content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                        index: 0,
                        id: Some("call_xyz".into()),
                        r#type: Some("function".into()),
                        function: Some(DeltaFunction {
                            name: Some("calculator".into()),
                            arguments: Some("{\"expr\":".into()),
                        }),
                        signature: None,
                    })]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::ToolCall(DeltaToolCall {
                        index: 0,
                        id: None,
                        r#type: None,
                        function: Some(DeltaFunction {
                            name: None,
                            arguments: Some("\"1+1\"}".into()),
                        }),
                        signature: None,
                    })]),
                    finish_reason: Some(FinishReason::ToolCalls),
                    ..Default::default()
                }),
            ];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ToolCallStart {
                        index: 0,
                        id: Some("call_xyz".into()),
                        r#type: Some("function".into()),
                        name: Some("calculator".into()),
                    },
                    ChatStreamEvent::ToolCallFinished(crate::tool::ToolCall {
                        id: "call_xyz".into(),
                        name: "calculator".into(),
                        arguments: serde_json::json!({ "expr": "1+1" }),
                        signature: None,
                    }),
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: Some(FinishReason::ToolCalls),
                    }
                ]
            );
        });
    }

    #[test]
    fn test_chat_stream_reasoning_lifecycle() {
        block_on(async {
            let deltas = vec![
                Ok::<_, String>(DeltaMessage {
                    role: Some(Role::Assistant),
                    content: Some(vec![DeltaContentPart::Reasoning {
                        index: 0,
                        text: "thinking part 1".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::Reasoning {
                        index: 0,
                        text: "thinking part 2".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
                Ok::<_, String>(DeltaMessage {
                    content: Some(vec![DeltaContentPart::Text {
                        index: 1,
                        text: "final answer".to_string(),
                        signature: None,
                    }]),
                    ..Default::default()
                }),
            ];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ReasoningStart,
                    ChatStreamEvent::Reasoning {
                        text: "thinking part 1".to_string()
                    },
                    ChatStreamEvent::Reasoning {
                        text: "thinking part 2".to_string()
                    },
                    ChatStreamEvent::ReasoningEnd,
                    ChatStreamEvent::Text {
                        text: "final answer".to_string()
                    },
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: None,
                    }
                ]
            );
        });
    }

    #[test]
    fn test_chat_stream_reasoning_end_on_stream_finish() {
        block_on(async {
            let deltas = vec![Ok::<_, String>(DeltaMessage {
                role: Some(Role::Assistant),
                content: Some(vec![DeltaContentPart::Reasoning {
                    index: 0,
                    text: "only reasoning".to_string(),
                    signature: None,
                }]),
                ..Default::default()
            })];

            let stream = stream::iter(deltas);
            let mut chat_stream = ChatStream::new(stream);

            let mut events = Vec::new();
            while let Some(res) = chat_stream.next().await {
                events.push(res.unwrap());
            }

            assert_eq!(
                events,
                vec![
                    ChatStreamEvent::Start {
                        role: Role::Assistant,
                        name: None,
                    },
                    ChatStreamEvent::ReasoningStart,
                    ChatStreamEvent::Reasoning {
                        text: "only reasoning".to_string()
                    },
                    ChatStreamEvent::ReasoningEnd,
                    ChatStreamEvent::Finished {
                        usage: None,
                        finish_reason: None,
                    }
                ]
            );
        });
    }
}
