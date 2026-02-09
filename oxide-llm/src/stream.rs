use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::BytesMut;
use futures::{Stream, ready};
use oxide_llm_core::message::DeltaMessage;
use oxide_llm_core::transport::TransportError;

use crate::error::{AgentError, Result};

/// A generic stream for processing Server-Sent Events (SSE) or similar framed streams.
///
/// 通用的 SSE 或者是类似分帧流的处理流。
pub struct MessageStream<S, F> {
    stream: S,
    buffer: BytesMut,
    stopped: bool,
    processor: F,
}

impl<S, F> MessageStream<S, F> {
    pub fn new(stream: S, processor: F) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            stopped: false,
            processor,
        }
    }
}

impl<S, F> Stream for MessageStream<S, F>
where
    S: Stream<Item = std::result::Result<bytes::Bytes, TransportError>> + Unpin,
    F: Fn(&[u8]) -> (Option<Result<DeltaMessage>>, bool) + Unpin,
{
    type Item = Result<DeltaMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 1. Process buffer
            if let Some(pos) = self.buffer.windows(2).position(|w| w == b"\n\n") {
                let block = self.buffer.split_to(pos + 2);
                let processor = &self.processor;
                let (item, stop) = processor(&block);

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
