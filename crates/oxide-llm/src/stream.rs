use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::BytesMut;
use futures::{Stream, ready};
use oxide_llm_core::message::{ChatStream, DeltaMessage};
use oxide_llm_core::transport::TransportError;

use crate::config::{DeltaHook, RawDeltaHook};
use crate::error::{AgentError, Result};
use tracing::{debug, warn};

/// Helper function to parse standard JSON SSE blocks into typed stream items.
///
/// 将标准 JSON SSE 数据块解析为强类型流项的辅助函数。
pub fn parse_json_sse_block<T, F>(
    block: &[u8],
    mut is_terminal_event: F,
) -> (Option<Result<T>>, bool)
where
    T: serde::de::DeserializeOwned,
    F: FnMut(&T) -> bool,
{
    let s = match std::str::from_utf8(block) {
        Ok(s) => s,
        Err(e) => {
            warn!("UTF-8 decode error in SSE block: {}", e);
            return (Some(Err(AgentError::Utf8(e))), false);
        }
    };

    debug!("Processing SSE raw block: {:?}", s);

    let mut chunk_to_yield = None;
    let mut done = false;
    let mut has_sse_header = false;
    let mut has_invalid_line = false;

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        if line.starts_with("event:") || line.starts_with("id:") || line.starts_with("retry:") {
            has_sse_header = true;
            continue;
        }

        if let Some(data) = line.strip_prefix("data:") {
            has_sse_header = true;
            let data = data.trim();
            if data == "[DONE]" {
                done = true;
                break;
            }
            match serde_json::from_str::<T>(data) {
                Ok(item) => {
                    if is_terminal_event(&item) {
                        done = true;
                    }
                    chunk_to_yield = Some(Ok(item));
                }
                Err(e) => {
                    warn!("Failed to parse SSE JSON: {}. Data: {}", e, data);
                    return (Some(Err(AgentError::Json(e))), false);
                }
            }
        } else {
            has_invalid_line = true;
        }
    }

    if has_invalid_line && !has_sse_header {
        warn!("Unexpected non-SSE block: {}", s);
        return (
            Some(Err(AgentError::StreamData(format!(
                "Unexpected response block: {}",
                s.trim()
            )))),
            false,
        );
    }

    (chunk_to_yield, done)
}

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

/// A generic stream for processing Server-Sent Events (SSE) or similar framed streams, with optional hooks.
///
/// 通用的 SSE 或者是类似分帧流的处理流（可选带有 Hook）。
pub struct MessageStream<S, P, Item = DeltaMessage> {
    stream: S,
    buffer: BytesMut,
    stopped: bool,
    processor: P,
    on_raw_delta: Option<RawDeltaHook<Item>>,
    _phantom: PhantomData<fn() -> Item>,
}

impl<S, P, Item> MessageStream<S, P, Item> {
    pub fn new(stream: S, processor: P) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            stopped: false,
            processor,
            on_raw_delta: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_hook(stream: S, processor: P, on_raw_delta: Option<RawDeltaHook<Item>>) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            stopped: false,
            processor,
            on_raw_delta,
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
                        if let Ok(ref val) = item
                            && let Some(hook) = self.on_raw_delta.as_mut()
                        {
                            hook(val);
                        }
                        return Poll::Ready(Some(item));
                    }
                    return Poll::Ready(None);
                }

                if let Some(item) = item {
                    if let Ok(ref val) = item
                        && let Some(hook) = self.on_raw_delta.as_mut()
                    {
                        hook(val);
                    }
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
    on_raw_delta: Option<RawDeltaHook<Item>>,
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
            on_raw_delta: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `AgentChatStreamRawFuture` with `on_raw_delta` hook.
    ///
    /// 创建带有 `on_raw_delta` Hook 的 `AgentChatStreamRawFuture`。
    pub fn with_hook(
        fut: Result<Fut>,
        processor: P,
        on_raw_delta: Option<RawDeltaHook<Item>>,
    ) -> Self {
        Self {
            fut,
            processor: Some(processor),
            on_raw_delta,
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
                        let on_raw_delta = this.on_raw_delta.take();
                        let message_stream =
                            MessageStream::with_hook(stream, processor, on_raw_delta);
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

/// A wrapper stream that triggers an `on_raw_delta` hook when items arrive.
///
/// 触发 `on_raw_delta` Hook 的流包装器。
pub struct RawHookStream<S, RawDelta> {
    stream: S,
    on_raw_delta: Option<RawDeltaHook<RawDelta>>,
}

impl<S, RawDelta> RawHookStream<S, RawDelta> {
    /// Creates a new `RawHookStream`.
    pub fn new(stream: S, on_raw_delta: Option<RawDeltaHook<RawDelta>>) -> Self {
        Self {
            stream,
            on_raw_delta,
        }
    }
}

impl<S, RawDelta> Stream for RawHookStream<S, RawDelta>
where
    S: Stream<Item = Result<RawDelta>> + Unpin,
{
    type Item = Result<RawDelta>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(raw))) => {
                if let Some(hook) = self.on_raw_delta.as_mut() {
                    hook(&raw);
                }
                Poll::Ready(Some(Ok(raw)))
            }
            res => res,
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

/// A stream wrapper that maps raw SSE items into DeltaMessages using a stream mapper, with optional hooks.
///
/// 使用流映射器将 Raw SSE 项映射为 DeltaMessage 的流包装器（可带 Hook）。
pub struct MappedStream<S, M, RawDelta = ()> {
    stream: S,
    mapper: M,
    on_raw_delta: Option<RawDeltaHook<RawDelta>>,
    on_delta: Option<DeltaHook>,
}

impl<S, M, RawDelta> MappedStream<S, M, RawDelta> {
    /// Creates a new `MappedStream`.
    ///
    /// 创建一个新的 `MappedStream`。
    pub fn new(stream: S, mapper: M) -> Self {
        Self {
            stream,
            mapper,
            on_raw_delta: None,
            on_delta: None,
        }
    }

    /// Creates a new `MappedStream` with hooks.
    ///
    /// 创建带有 Hook 的 `MappedStream`。
    pub fn with_hooks(
        stream: S,
        mapper: M,
        on_raw_delta: Option<RawDeltaHook<RawDelta>>,
        on_delta: Option<DeltaHook>,
    ) -> Self {
        Self {
            stream,
            mapper,
            on_raw_delta,
            on_delta,
        }
    }
}

impl<S, M, RawDelta> Stream for MappedStream<S, M, RawDelta>
where
    S: Stream<Item = Result<RawDelta>> + Unpin,
    M: StreamMapper<RawDelta>,
{
    type Item = Result<DeltaMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(raw))) => {
                    if let Some(hook) = self.on_raw_delta.as_mut() {
                        hook(&raw);
                    }
                    match self.mapper.map_item(raw) {
                        Ok(Some(delta)) => {
                            if let Some(hook) = self.on_delta.as_mut() {
                                hook(&delta);
                            }
                            return Poll::Ready(Some(Ok(delta)));
                        }
                        Ok(None) => continue,
                        Err(e) => return Poll::Ready(Some(Err(e))),
                    }
                }
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
pub struct AgentChatStreamFuture<Fut, M, RawDelta = ()> {
    fut: Fut,
    mapper: Option<M>,
    on_raw_delta: Option<RawDeltaHook<RawDelta>>,
    on_delta: Option<DeltaHook>,
}

impl<Fut, M, RawDelta> AgentChatStreamFuture<Fut, M, RawDelta> {
    /// Creates a new `AgentChatStreamFuture`.
    ///
    /// 创建一个新的 `AgentChatStreamFuture`。
    pub fn new(fut: Fut, mapper: M) -> Self {
        Self {
            fut,
            mapper: Some(mapper),
            on_raw_delta: None,
            on_delta: None,
        }
    }

    /// Creates a new `AgentChatStreamFuture` with hooks.
    ///
    /// 创建带有 Hook 的 `AgentChatStreamFuture`。
    pub fn with_hooks(
        fut: Fut,
        mapper: M,
        on_raw_delta: Option<RawDeltaHook<RawDelta>>,
        on_delta: Option<DeltaHook>,
    ) -> Self {
        Self {
            fut,
            mapper: Some(mapper),
            on_raw_delta,
            on_delta,
        }
    }
}

impl<Fut, S, M, RawDelta> Future for AgentChatStreamFuture<Fut, M, RawDelta>
where
    Fut: Future<Output = Result<S>>,
    S: Stream<Item = Result<RawDelta>> + Unpin + Send + 'static,
    M: StreamMapper<RawDelta>,
{
    type Output = Result<ChatStream<MappedStream<S, M, RawDelta>, AgentError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let fut_pin = unsafe { Pin::new_unchecked(&mut this.fut) };
        match fut_pin.poll(cx) {
            Poll::Ready(Ok(raw_stream)) => {
                let mapper = this
                    .mapper
                    .take()
                    .expect("AgentChatStreamFuture polled after completion");
                let on_raw_delta = this.on_raw_delta.take();
                let on_delta = this.on_delta.take();
                let mapped_stream =
                    MappedStream::with_hooks(raw_stream, mapper, on_raw_delta, on_delta);
                Poll::Ready(Ok(ChatStream::new(mapped_stream)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DummyEvent {
        val: String,
        stop: Option<bool>,
    }

    #[test]
    fn test_parse_json_sse_block_valid() {
        let block = b"data: {\"val\":\"hello\"}\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |_| false);
        assert!(!done);
        assert!(item.is_some());
        let event = item.unwrap().unwrap();
        assert_eq!(
            event,
            DummyEvent {
                val: "hello".into(),
                stop: None
            }
        );
    }

    #[test]
    fn test_parse_json_sse_block_done_keyword() {
        let block = b"data: [DONE]\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |_| false);
        assert!(done);
        assert!(item.is_none());
    }

    #[test]
    fn test_parse_json_sse_block_terminal_closure() {
        let block = b"data: {\"val\":\"end\",\"stop\":true}\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |ev| ev.stop == Some(true));
        assert!(done);
        assert!(item.is_some());
    }

    #[test]
    fn test_parse_json_sse_block_unexpected_html() {
        let block = b"<!doctype html>\n<html lang=\"en\">\n<head></head>\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |_| false);
        assert!(!done);
        assert!(item.is_some());
        let err = item.unwrap().unwrap_err();
        if let AgentError::StreamData(msg) = err {
            assert!(msg.contains("<!doctype html>"));
        } else {
            panic!("Expected AgentError::StreamData, got {:?}", err);
        }
    }

    #[test]
    fn test_parse_json_sse_block_invalid_json() {
        let block = b"data: {invalid json}\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |_| false);
        assert!(!done);
        assert!(item.is_some());
        let err = item.unwrap().unwrap_err();
        assert!(matches!(err, AgentError::Json(_)));
    }

    #[test]
    fn test_parse_json_sse_block_comments_and_empty_lines() {
        let block = b": keep-alive\n: comment line\n\n";
        let (item, done) = parse_json_sse_block::<DummyEvent, _>(block, |_| false);
        assert!(!done);
        assert!(item.is_none());
    }
}
