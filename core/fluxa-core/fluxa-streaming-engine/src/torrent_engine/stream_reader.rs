use std::pin::Pin;
use std::task::{Context, Poll};

use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

use super::debug_log;

pub(super) struct CancellableReader<R> {
    inner: R,
    cancellation: Pin<Box<WaitForCancellationFutureOwned>>,
}

pub(super) struct TrackedReader<R> {
    inner: R,
    received: u64,
    expected: u64,
    torrent_id: usize,
    file_id: usize,
    range_start: u64,
}

impl<R> TrackedReader<R> {
    pub(super) fn new(
        inner: R,
        expected: u64,
        torrent_id: usize,
        file_id: usize,
        range_start: u64,
    ) -> Self {
        Self {
            inner,
            received: 0,
            expected,
            torrent_id,
            file_id,
            range_start,
        }
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for TrackedReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let added = (buf.filled().len() - before) as u64;
                self.received = self.received.saturating_add(added);
                if added == 0 {
                    debug_log(format!(
                        "[TorrServer][stream] eof torrent={} file={} start={} received={} expected={}",
                        self.torrent_id,
                        self.file_id,
                        self.range_start,
                        self.received,
                        self.expected
                    ));
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => {
                debug_log(format!(
                    "[TorrServer][stream] read_error torrent={} file={} start={} received={} expected={} error={error}",
                    self.torrent_id, self.file_id, self.range_start, self.received, self.expected
                ));
                Poll::Ready(Err(error))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<R> CancellableReader<R> {
    pub(super) fn new(inner: R, cancel: CancellationToken) -> Self {
        Self {
            inner,
            cancellation: Box::pin(cancel.cancelled_owned()),
        }
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for CancellableReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.cancellation.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "playback session cancelled",
            )));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
