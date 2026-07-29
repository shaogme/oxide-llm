use crate::ChatAgent;
use crate::core::message::{ChatStream, ChatStreamEvent, ContentPart, Message, Role};
use crate::core::state::ConversationState;
use crate::core::tool::{ToolCall, ToolResult};
use crate::error::AgentError;
use crate::tool::ToolRegistry;
use futures::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Future that executes a series of tool calls sequentially.
///
/// 顺序执行一系列工具调用的 Future。
pub struct ExecuteToolsFuture {
    registry: ToolRegistry,
    tool_calls: std::vec::IntoIter<ToolCall>,
    current_exec: Option<(ToolCall, oxide_llm_core::tool::ToolFuture)>,
    results: Vec<ToolResult>,
}

impl ExecuteToolsFuture {
    /// Creates a new `ExecuteToolsFuture`.
    ///
    /// 创建一个新的 `ExecuteToolsFuture`。
    pub fn new(registry: ToolRegistry, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            registry,
            tool_calls: tool_calls.into_iter(),
            current_exec: None,
            results: Vec::new(),
        }
    }
}

impl Unpin for ExecuteToolsFuture {}

impl Future for ExecuteToolsFuture {
    type Output = Vec<ToolResult>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        loop {
            if let Some((_, ref mut fut)) = this.current_exec {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(res_bucket) => {
                        let (tool_call, _) = this.current_exec.take().unwrap();
                        let result = match res_bucket {
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
                        };
                        this.results.push(result);
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            if let Some(tool_call) = this.tool_calls.next() {
                if let Some(fut) =
                    this.registry.execute_future(&tool_call.name, tool_call.arguments.clone())
                {
                    this.current_exec = Some((tool_call, fut));
                } else {
                    let result = ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        name: tool_call.name.clone(),
                        content: vec![ContentPart::Text {
                            text: format!("Error: Unknown tool '{}'", tool_call.name),
                            signature: None,
                        }],
                        is_error: true,
                        signature: tool_call.signature.clone(),
                    };
                    this.results.push(result);
                }
            } else {
                return Poll::Ready(std::mem::take(&mut this.results));
            }
        }
    }
}

/// A stream that manages the agent interaction loop, including tool execution.
pub struct RunnerStream<'a, A: ChatAgent + ?Sized + 'a> {
    agent: &'a A,
    registry: &'a ToolRegistry,
    state: &'a mut ConversationState,
    max_turns: usize,

    phase: Phase<'a, A>,
    current_turn: usize,
    collected_events: Vec<ChatStreamEvent>,
}

enum Phase<'a, A: ChatAgent + ?Sized + 'a> {
    Start,
    Initializing(A::ChatStreamFuture<'a>),
    Streaming(ChatStream<A::Stream, AgentError>),
    ExecutingTools(ExecuteToolsFuture),
    Done,
}

impl<'a, A: ChatAgent + ?Sized + 'a> RunnerStream<'a, A> {
    pub fn new(
        agent: &'a A,
        registry: &'a ToolRegistry,
        state: &'a mut ConversationState,
        max_turns: usize,
    ) -> Self {
        RunnerStream {
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

impl<'a, A> Stream for RunnerStream<'a, A>
where
    A: ChatAgent + ?Sized + 'a,
    A::Stream: Unpin,
    A::ChatStreamFuture<'a>: Unpin,
{
    type Item = Result<ChatStreamEvent, AgentError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        loop {
            match &mut this.phase {
                Phase::Start => {
                    if this.current_turn >= this.max_turns {
                        this.phase = Phase::Done;
                        return Poll::Ready(None);
                    }

                    this.current_turn += 1;
                    this.collected_events.clear();

                    let state_clone = this.state.clone();
                    let fut = this.agent.chat_stream(state_clone);
                    this.phase = Phase::Initializing(fut);
                }
                Phase::Initializing(fut) => {
                    match Pin::new(fut).poll(cx) {
                        Poll::Ready(Ok(stream)) => {
                            this.phase = Phase::Streaming(stream);
                        }
                        Poll::Ready(Err(e)) => {
                            this.phase = Phase::Done;
                            return Poll::Ready(Some(Err(e)));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::Streaming(stream) => {
                    match Pin::new(stream).poll_next(cx) {
                        Poll::Ready(Some(Ok(event))) => {
                            this.collected_events.push(event.clone());
                            return Poll::Ready(Some(Ok(event)));
                        }
                        Poll::Ready(Some(Err(e))) => {
                            return Poll::Ready(Some(Err(e)));
                        }
                        Poll::Ready(None) => {
                            let message: Message = this.collected_events.drain(..).collect();
                            this.state.add_message(message.clone());

                            let tool_calls: Vec<ToolCall> = message
                                .content
                                .into_iter()
                                .filter_map(|part| match part {
                                    ContentPart::ToolCall(tc) => Some(tc),
                                    _ => None,
                                })
                                .collect();

                            if tool_calls.is_empty() {
                                this.phase = Phase::Done;
                                return Poll::Ready(None);
                            }

                            let exec_fut = ExecuteToolsFuture::new(this.registry.clone(), tool_calls);
                            this.phase = Phase::ExecutingTools(exec_fut);
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::ExecutingTools(fut) => {
                    match Pin::new(fut).poll(cx) {
                        Poll::Ready(results) => {
                            this.state.add_message(Message {
                                role: Role::Tool,
                                content: results.into_iter().map(ContentPart::ToolResult).collect(),
                                name: None,
                            });

                            this.phase = Phase::Start;
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Phase::Done => return Poll::Ready(None),
            }
        }
    }
}
