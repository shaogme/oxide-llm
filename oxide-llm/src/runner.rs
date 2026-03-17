use crate::DynChatAgent;
use crate::core::message::{ChatStreamEvent, ContentPart, Message, Role};
use crate::core::state::ConversationState;
use crate::core::tool::{ToolCall, ToolResult};
use crate::error::AgentError;
use crate::tool::ToolRegistry;
use futures::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A stream that manages the agent interaction loop, including tool execution.
pub struct RunnerStream<'a> {
    agent: &'a dyn DynChatAgent,
    registry: &'a ToolRegistry,
    state: &'a mut ConversationState,
    max_turns: usize,

    phase: Phase<'a>,
    current_turn: usize,
    collected_events: Vec<ChatStreamEvent>,
}

enum Phase<'a> {
    Start,
    Initializing(
        futures::future::BoxFuture<
            'a,
            Result<
                crate::core::message::ChatStream<
                    futures::stream::BoxStream<
                        'static,
                        Result<crate::core::message::DeltaMessage, AgentError>,
                    >,
                    AgentError,
                >,
                AgentError,
            >,
        >,
    ),
    Streaming(Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, AgentError>> + Send + 'a>>),
    ExecutingTools(Pin<Box<dyn Future<Output = Vec<ToolResult>> + Send + 'a>>),
    Done,
}

impl<'a> RunnerStream<'a> {
    pub fn new(
        agent: &'a dyn DynChatAgent,
        registry: &'a ToolRegistry,
        state: &'a mut ConversationState,
        max_turns: usize,
    ) -> Self {
        Self {
            agent,
            registry,
            state,
            max_turns,
            phase: Phase::Start,
            current_turn: 0,
            collected_events: Vec::new(),
        }
    }
}

impl<'a> Stream for RunnerStream<'a> {
    type Item = Result<ChatStreamEvent, AgentError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &mut self.phase {
                Phase::Start => {
                    if self.current_turn >= self.max_turns {
                        self.phase = Phase::Done;
                        return Poll::Ready(None);
                    }

                    self.current_turn += 1;
                    self.collected_events.clear();

                    // Start the chat stream
                    // We need to clone the state because chat_stream consumes it,
                    // but we need to keep our mutable reference for updates.
                    let state_clone = self.state.clone();

                    // Create a future that resolves to the stream and put it in Phase::Initializing
                    let fut = self.agent.chat_stream(state_clone);
                    self.phase = Phase::Initializing(fut);
                }
                Phase::Initializing(fut) => {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(Ok(stream)) => {
                            // Convert the ChatStream (which is a wrapper) into a BoxStream of events
                            // Note: The ChatStream implements Stream<Item = Result<ChatStreamEvent, E>>
                            // We need to box it to store in Phase enum
                            self.phase = Phase::Streaming(Box::pin(stream));
                        }
                        Poll::Ready(Err(e)) => {
                            self.phase = Phase::Done;
                            return Poll::Ready(Some(Err(e)));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::Streaming(stream) => {
                    match stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(Ok(event))) => {
                            self.collected_events.push(event.clone());
                            return Poll::Ready(Some(Ok(event)));
                        }
                        Poll::Ready(Some(Err(e))) => {
                            // On error, expose it? Or stop?
                            return Poll::Ready(Some(Err(e)));
                        }
                        Poll::Ready(None) => {
                            // Stream finished. Reconstruct message and check for tools.
                            let message: Message = self.collected_events.drain(..).collect();
                            self.state.add_message(message.clone());

                            let tool_calls: Vec<ToolCall> = message
                                .content
                                .into_iter()
                                .filter_map(|part| match part {
                                    ContentPart::ToolCall(tc) => Some(tc),
                                    _ => None,
                                })
                                .collect();

                            if tool_calls.is_empty() {
                                self.phase = Phase::Done;
                                return Poll::Ready(None);
                            }

                            // Prepare tool execution
                            let registry = self.registry.clone();
                            let fut = async move {
                                let mut results = Vec::new();
                                for tool_call in tool_calls {
                                    let result = if let Some(res_bucket) = registry
                                        .execute(&tool_call.name, tool_call.arguments.clone())
                                        .await
                                    {
                                        match res_bucket {
                                            Ok(content) => ToolResult {
                                                tool_call_id: tool_call.id.clone(),
                                                name: tool_call.name.clone(),
                                                content,
                                                is_error: false,
                                                signature: tool_call.signature.clone(),
                                            },
                                            Err(err) => ToolResult {
                                                tool_call_id: tool_call.id.clone(),
                                                name: tool_call.name.clone(),
                                                content: vec![ContentPart::Text {
                                                    text: format!("Error executing tool: {}", err),
                                                    signature: None,
                                                }],
                                                is_error: true,
                                                signature: tool_call.signature.clone(),
                                            },
                                        }
                                    } else {
                                        ToolResult {
                                            tool_call_id: tool_call.id.clone(),
                                            name: tool_call.name.clone(),
                                            content: vec![ContentPart::Text {
                                                text: format!(
                                                    "Error: Unknown tool '{}'",
                                                    tool_call.name
                                                ),
                                                signature: None,
                                            }],
                                            is_error: true,
                                            signature: tool_call.signature.clone(),
                                        }
                                    };
                                    results.push(result);
                                }
                                results
                            };

                            self.phase = Phase::ExecutingTools(Box::pin(fut));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::ExecutingTools(fut) => {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(results) => {
                            // Add tool results to state
                            self.state.add_message(Message {
                                role: Role::Tool,
                                content: results.into_iter().map(ContentPart::ToolResult).collect(),
                                name: None,
                            });

                            // Loop back to Start for next turn
                            self.phase = Phase::Start;
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::Done => return Poll::Ready(None),
            }
        }
    }
}
