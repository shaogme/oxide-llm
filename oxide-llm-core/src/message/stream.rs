use futures::{Stream, StreamExt};
use ref_str::StaticRefStr;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::message::assembler::MessageAssembler;
use crate::message::delta::{DeltaContentPart, DeltaMessage, FinishReason, Usage};
use crate::message::model::{ContentPart, Message, Role};
use crate::tool::ToolCall;

/// Event emitted by ChatStream.
///
/// ChatStream 产生的事件。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamEvent {
    /// Conversation start.
    ///
    /// 对话开始。
    Start {
        role: Role,
        name: Option<StaticRefStr>,
    },

    /// Text stream chunk.
    ///
    /// 文本流片段。
    Text { text: String },

    /// Reasoning block start.
    ///
    /// 思考/推理块开始。
    ReasoningStart,

    /// Reasoning stream chunk.
    ///
    /// 思考/推理流片段。
    Reasoning { text: String },

    /// Reasoning block end.
    ///
    /// 思考/推理块结束。
    ReasoningEnd,

    /// Tool call start chunk.
    ///
    /// 工具调用开始。
    ToolCallStart {
        index: u32,
        id: Option<StaticRefStr>,
        #[serde(rename = "tool_type")]
        r#type: Option<StaticRefStr>,
        name: Option<StaticRefStr>,
    },

    /// Tool call complete.
    ///
    /// 工具调用完成。
    ToolCallFinished(ToolCall),

    /// Response finished.
    ///
    /// 响应结束。
    Finished {
        usage: Option<Usage>,
        finish_reason: Option<FinishReason>,
    },
}

impl FromIterator<ChatStreamEvent> for Message {
    fn from_iter<T: IntoIterator<Item = ChatStreamEvent>>(iter: T) -> Self {
        let mut role = Role::Assistant;
        let mut name = None;
        let mut content = Vec::new();

        for event in iter {
            match event {
                ChatStreamEvent::Start { role: r, name: n } => {
                    role = r;
                    name = n;
                }
                ChatStreamEvent::Text { text } => match content.last_mut() {
                    Some(ContentPart::Text {
                        text: current,
                        signature: _,
                    }) => {
                        current.push_str(&text);
                    }
                    _ => {
                        content.push(ContentPart::Text {
                            text,
                            signature: None,
                        });
                    }
                },
                ChatStreamEvent::Reasoning { text } => match content.last_mut() {
                    Some(ContentPart::Reasoning { text: current, .. }) => {
                        current.push_str(&text);
                    }
                    _ => {
                        content.push(ContentPart::Reasoning {
                            text,
                            signature: None,
                        });
                    }
                },
                ChatStreamEvent::ToolCallFinished(tool_call) => {
                    content.push(ContentPart::ToolCall(tool_call));
                }
                _ => {}
            }
        }

        Message {
            role,
            content,
            name,
        }
    }
}

/// A stream wrapper that converts DeltaMessages into high-level ChatStreamEvents
/// and automatically assembles the final Message.
///
/// 一个流包装器，将 DeltaMessages 转换为高级 ChatStreamEvents，并自动组装最终的 Message。
pub struct ChatStream<S, E> {
    stream: S,
    assembler: MessageAssembler,
    pending_events: VecDeque<ChatStreamEvent>,
    stream_finished: bool,
    start_event_emitted: bool,
    finished_event_emitted: bool,
    in_reasoning: bool,
    emitted_tool_starts: HashSet<u32>,
    emitted_tool_finishes: HashSet<u32>,
    _marker: PhantomData<E>,
}

impl<S, E> ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Send + 'static,
{
    pub fn into_boxed(
        self,
    ) -> ChatStream<futures::stream::BoxStream<'static, Result<DeltaMessage, E>>, E> {
        ChatStream {
            stream: self.stream.boxed(),
            assembler: self.assembler,
            pending_events: self.pending_events,
            stream_finished: self.stream_finished,
            start_event_emitted: self.start_event_emitted,
            finished_event_emitted: self.finished_event_emitted,
            in_reasoning: self.in_reasoning,
            emitted_tool_starts: self.emitted_tool_starts,
            emitted_tool_finishes: self.emitted_tool_finishes,
            _marker: PhantomData,
        }
    }
}

impl<S, E> ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            assembler: MessageAssembler::new(),
            pending_events: VecDeque::new(),
            stream_finished: false,
            start_event_emitted: false,
            finished_event_emitted: false,
            in_reasoning: false,
            emitted_tool_starts: HashSet::new(),
            emitted_tool_finishes: HashSet::new(),
            _marker: PhantomData,
        }
    }
}

impl<S, E> Stream for ChatStream<S, E>
where
    S: Stream<Item = Result<DeltaMessage, E>> + Unpin,
    E: std::fmt::Debug + Unpin,
{
    type Item = Result<ChatStreamEvent, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            // 1. Drain pending events first
            if let Some(event) = this.pending_events.pop_front() {
                return Poll::Ready(Some(Ok(event)));
            }

            // 2. If finished event emitted, we are done
            if this.finished_event_emitted {
                return Poll::Ready(None);
            }

            // 3. If stream finished but we haven't emitted Finished event, do it now
            if this.stream_finished {
                if this.in_reasoning {
                    this.in_reasoning = false;
                    this.pending_events.push_back(ChatStreamEvent::ReasoningEnd);
                    continue;
                }

                // Emit all completed tool calls that haven't been emitted yet
                let tool_indices = this.assembler.get_tool_call_indices();
                for index in tool_indices {
                    if !this.emitted_tool_finishes.contains(&index)
                        && let Some(tool_call) = this.assembler.get_tool_call(index)
                    {
                        this.pending_events
                            .push_back(ChatStreamEvent::ToolCallFinished(tool_call));
                        this.emitted_tool_finishes.insert(index);
                    }
                }

                // Drain pending events before emitting Finished
                if let Some(event) = this.pending_events.pop_front() {
                    return Poll::Ready(Some(Ok(event)));
                }

                this.finished_event_emitted = true;

                let usage = this.assembler.usage();
                let finish_reason = this.assembler.finish_reason();

                return Poll::Ready(Some(Ok(ChatStreamEvent::Finished {
                    usage,
                    finish_reason,
                })));
            }

            // 4. Poll underlying stream
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(mut delta))) => {
                    // 4.1 Extract content and update metadata directly
                    let content = delta.content.take();
                    this.assembler.add_metadata(&delta);

                    // 4.2 Emit Start Event if not yet emitted
                    if !this.start_event_emitted {
                        this.start_event_emitted = true;
                        this.pending_events.push_back(ChatStreamEvent::Start {
                            role: this.assembler.role().unwrap_or(Role::Assistant),
                            name: this.assembler.name(),
                        });
                    }

                    // 4.3 Process Content and feed to Assembler directly
                    if let Some(parts) = content {
                        for part in parts {
                            match &part {
                                DeltaContentPart::Text { text, .. } => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Text {
                                            text: text.clone(),
                                        });
                                    }
                                }
                                DeltaContentPart::Reasoning { text, .. } => {
                                    if !this.in_reasoning {
                                        this.in_reasoning = true;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningStart);
                                    }
                                    if !text.is_empty() {
                                        this.pending_events.push_back(ChatStreamEvent::Reasoning {
                                            text: text.clone(),
                                        });
                                    }
                                }
                                DeltaContentPart::ToolCall(tool) => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
                                    if !this.emitted_tool_starts.contains(&tool.index) {
                                        this.emitted_tool_starts.insert(tool.index);
                                        this.pending_events.push_back(
                                            ChatStreamEvent::ToolCallStart {
                                                index: tool.index,
                                                id: tool.id.clone(),
                                                r#type: tool.r#type.clone(),
                                                name: tool
                                                    .function
                                                    .as_ref()
                                                    .and_then(|f| f.name.clone()),
                                            },
                                        );
                                    }
                                }
                                DeltaContentPart::Refusal { .. } => {
                                    if this.in_reasoning {
                                        this.in_reasoning = false;
                                        this.pending_events
                                            .push_back(ChatStreamEvent::ReasoningEnd);
                                    }
                                }
                            }
                            this.assembler.add_part(part);
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    this.stream_finished = true;
                    // Continue loop to hit step 3
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
