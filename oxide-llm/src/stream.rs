use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::BytesMut;
use futures::{Stream, ready};
use oxide_llm_core::message::DeltaMessage;
use oxide_llm_core::transport::TransportError;

use crate::error::{AgentError, Result};

/// Trait for processing SSE data blocks into delta messages.
///
/// 将 SSE 数据块处理为增量消息的 Trait。
pub trait SseProcessor: Unpin {
    /// Process a data block and return an optional `DeltaMessage` result along with a stop flag.
    ///
    /// 处理一个数据块并返回可选的 `DeltaMessage` 结果以及停止标志。
    fn process(&mut self, block: &[u8]) -> (Option<Result<DeltaMessage>>, bool);
}

impl<F> SseProcessor for F
where
    F: FnMut(&[u8]) -> (Option<Result<DeltaMessage>>, bool) + Unpin,
{
    fn process(&mut self, block: &[u8]) -> (Option<Result<DeltaMessage>>, bool) {
        (self)(block)
    }
}

/// A generic stream for processing Server-Sent Events (SSE) or similar framed streams.
///
/// 通用的 SSE 或者是类似分帧流的处理流。
pub struct MessageStream<S, P> {
    stream: S,
    buffer: BytesMut,
    stopped: bool,
    processor: P,
}

impl<S, P> MessageStream<S, P> {
    pub fn new(stream: S, processor: P) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            stopped: false,
            processor,
        }
    }
}

impl<S, P> Stream for MessageStream<S, P>
where
    S: Stream<Item = std::result::Result<bytes::Bytes, TransportError>> + Unpin,
    P: SseProcessor,
{
    type Item = Result<DeltaMessage>;

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
