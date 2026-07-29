use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::BytesMut;
use futures::{ready, Stream};
use oxide_llm_core::message::{ChatStream, DeltaMessage};
use oxide_llm_core::transport::TransportError;

use crate::error::{AgentError, Result};

/// Trait for processing SSE data blocks into raw stream items.
///
/// 将 SSE 数据块处理为原始流项的 Trait。
pub trait SseProcessor<Item = DeltaMessage>: Unpin {
    /// Process a data block and return an optional `Item` result along with a stop flag.
    ///
    /// 处理一个数据块并返回可选的 `Item` 结果以及停止标志。
    fn process(&mut self, block: &[u8]) -> (Option<Result<Item>>, bool);
}

impl<F, Item> SseProcessor<Item> for F
where
    F: FnMut(&[u8]) -> (Option<Result<Item>>, bool) + Unpin,
{
    fn process(&mut self, block: &[u8]) -> (Option<Result<Item>>, bool) {
        (self)(block)
    }
}

/// A generic stream for processing Server-Sent Events (SSE) or similar framed streams.
///
/// 通用的 SSE 或者是类似分帧流的处理流。
pub struct MessageStream<S, P, Item = DeltaMessage> {
    stream: S,
    buffer: BytesMut,
    stopped: bool,
    processor: P,
    _phantom: PhantomData<fn() -> Item>,
}

impl<S, P, Item> MessageStream<S, P, Item> {
    pub fn new(stream: S, processor: P) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            stopped: false,
            processor,
            _phantom: PhantomData,
        }
    }
}

impl<S, P, Item> Stream for MessageStream<S, P, Item>
where
    S: Stream<Item = std::result::Result<bytes::Bytes, TransportError>> + Unpin,
    P: SseProcessor<Item>,
{
    type Item = Result<Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 1. Process buffer
            if let Some(pos) = self.buffer.windows(2).position(|w| w == b"\n\n") {
                let block = self.buffer.split_to(pos + 2);
                let (item, stop) = self.processor.process(&block);

                if stop {
                    self.stopped = true;
                    if let Some(item) = item {
                        return Poll::Ready(Some(item));
                    }
                    return Poll::Ready(None);
                }

                if let Some(item) = item {
                    return Poll::Ready(Some(item));
                }

                // If item is None (ignored/keep-alive) and not stopped, continue loop
                continue;
            }

            // 2. Poll stream
            match ready!(Pin::new(&mut self.stream).poll_next(cx)) {
                Some(Ok(chunk)) => {
                    self.buffer.extend_from_slice(&chunk);
                }
                Some(Err(e)) => {
                    return Poll::Ready(Some(Err(AgentError::Transport(e))));
                }
                None => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

/// A named concrete Future that maps a transport stream future into a raw `MessageStream`.
///
/// 将传输流 Future 映射为裸 `MessageStream` 的具名 Future 结构体。
pub struct AgentChatStreamRawFuture<Fut, P, Item = DeltaMessage> {
    fut: std::result::Result<Fut, AgentError>,
    processor: Option<P>,
    _phantom: PhantomData<fn() -> Item>,
}

impl<Fut, P, Item> AgentChatStreamRawFuture<Fut, P, Item> {
    /// Creates a new `AgentChatStreamRawFuture`.
    ///
    /// 创建一个新的 `AgentChatStreamRawFuture`。
    pub fn new(fut: Result<Fut>, processor: P) -> Self {
        Self {
            fut,
            processor: Some(processor),
            _phantom: PhantomData,
        }
    }
}

impl<Fut, S, P, Item> Future for AgentChatStreamRawFuture<Fut, P, Item>
where
    Fut: Future<Output = std::result::Result<S, TransportError>>,
    S: Stream<Item = std::result::Result<bytes::Bytes, TransportError>> + Unpin,
    P: SseProcessor<Item>,
{
    type Output = Result<MessageStream<S, P, Item>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        match this.fut.as_mut() {
            Ok(fut) => {
                let fut_pin = unsafe { Pin::new_unchecked(fut) };
                match fut_pin.poll(cx) {
                    Poll::Ready(Ok(stream)) => {
                        let processor = this
                            .processor
                            .take()
                            .expect("processor polled after completion");
                        let message_stream = MessageStream::new(stream, processor);
                        Poll::Ready(Ok(message_stream))
                    }
                    Poll::Ready(Err(e)) => Poll::Ready(Err(AgentError::Transport(e))),
                    Poll::Pending => Poll::Pending,
                }
            }
            Err(e) => {
                let err = AgentError::AlreadyPolled(
                    "AgentChatStreamRawFuture polled after completion or with initial error"
                        .to_string(),
                );
                Poll::Ready(Err(std::mem::replace(e, err)))
            }
        }
    }
}

/// Trait for mapping provider raw stream delta/event into core `DeltaMessage`.
///
/// 将 Provider 原始流增量/事件映射为核心 `DeltaMessage` 的 Trait。
pub trait StreamMapper<RawDelta>: Unpin {
    /// Map a raw item to an optional `DeltaMessage`.
    /// Returns `Ok(None)` if the event should be ignored.
    ///
    /// 将原始项映射为可选的 `DeltaMessage`。如果忽略该事件则返回 `Ok(None)`。
    fn map_item(&mut self, raw: RawDelta) -> Result<Option<DeltaMessage>>;
}

/// A stream wrapper that maps raw SSE items into DeltaMessages using a stream mapper.
///
/// 使用流映射器将 Raw SSE 项映射为 DeltaMessage 的流包装器。
pub struct MappedStream<S, M> {
    stream: S,
    mapper: M,
}

impl<S, M> MappedStream<S, M> {
    /// Creates a new `MappedStream`.
    ///
    /// 创建一个新的 `MappedStream`。
    pub fn new(stream: S, mapper: M) -> Self {
        Self { stream, mapper }
    }
}

impl<S, M, RawDelta> Stream for MappedStream<S, M>
where
    S: Stream<Item = Result<RawDelta>> + Unpin,
    M: StreamMapper<RawDelta>,
{
    type Item = Result<DeltaMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(raw))) => match self.mapper.map_item(raw) {
                    Ok(Some(delta)) => return Poll::Ready(Some(Ok(delta))),
                    Ok(None) => continue,
                    Err(e) => return Poll::Ready(Some(Err(e))),
                },
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// A named concrete Future that wraps a raw stream future and converts the raw stream into a `ChatStream`.
///
/// 包装裸流 Future 并将裸流转换为 `ChatStream` 的具名 Future 结构体。
pub struct AgentChatStreamFuture<Fut, M> {
    fut: Fut,
    mapper: Option<M>,
}

impl<Fut, M> AgentChatStreamFuture<Fut, M> {
    /// Creates a new `AgentChatStreamFuture`.
    ///
    /// 创建一个新的 `AgentChatStreamFuture`。
    pub fn new(fut: Fut, mapper: M) -> Self {
        Self {
            fut,
            mapper: Some(mapper),
        }
    }
}

impl<Fut, S, M, RawDelta> Future for AgentChatStreamFuture<Fut, M>
where
    Fut: Future<Output = Result<S>>,
    S: Stream<Item = Result<RawDelta>> + Unpin + Send + 'static,
    M: StreamMapper<RawDelta>,
{
    type Output = Result<ChatStream<MappedStream<S, M>, AgentError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let fut_pin = unsafe { Pin::new_unchecked(&mut this.fut) };
        match fut_pin.poll(cx) {
            Poll::Ready(Ok(raw_stream)) => {
                let mapper = this
                    .mapper
                    .take()
                    .expect("AgentChatStreamFuture polled after completion");
                let mapped_stream = MappedStream::new(raw_stream, mapper);
                Poll::Ready(Ok(ChatStream::new(mapped_stream)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
