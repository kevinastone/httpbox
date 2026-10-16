use futures::prelude::*;
use hyper::body::Bytes;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) trait AsByteSlice {
    fn as_byte_slice(&self) -> &[u8];
}

impl AsByteSlice for u8 {
    #[inline]
    fn as_byte_slice(&self) -> &[u8] {
        std::slice::from_ref(self)
    }
}

macro_rules! impl_as_byte_slice {
    ($($ty:ty => $method:ident),* $(,)?) => {
        $(
            impl AsByteSlice for $ty {
                #[inline]
                fn as_byte_slice(&self) -> &[u8] {
                    self.$method()
                }
            }
        )*
    };
}

impl_as_byte_slice! {
    &[u8] => as_ref,
    Vec<u8> => as_slice,
    Bytes => as_ref,
    &str => as_bytes,
    String => as_bytes,
}

pub(crate) trait StreamChunkBytesExt: Stream
where
    Self::Item: AsByteSlice,
{
    fn chunks_bytes(self, capacity: NonZeroUsize) -> ChunksBytes<Self>
    where
        Self: Sized,
    {
        ChunksBytes {
            stream: self,
            capacity,
            pending: Vec::new(),
            offset: 0,
        }
    }
}

impl<S: Stream + ?Sized> StreamChunkBytesExt for S where S::Item: AsByteSlice {}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub(crate) struct ChunksBytes<S> {
    stream: S,
    capacity: NonZeroUsize,
    pending: Vec<u8>,
    offset: usize,
}

impl<S> Stream for ChunksBytes<S>
where
    S: Stream + Unpin,
    S::Item: AsByteSlice,
{
    type Item = Bytes;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let capacity = self.capacity.get();

        let available = self.pending.len() - self.offset;
        if available >= capacity {
            let start = self.offset;
            let end = start + capacity;
            self.offset = end;
            return Poll::Ready(Some(Bytes::copy_from_slice(
                &self.pending[start..end],
            )));
        }

        while self.pending.len() - self.offset < capacity {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(chunk)) => {
                    let offset = self.offset;
                    if offset > 0 {
                        self.pending.drain(..offset);
                        self.offset = 0;
                    }
                    self.pending.extend_from_slice(chunk.as_byte_slice());
                }
                Poll::Ready(None) => break,
                Poll::Pending => {
                    if self.pending.len() - self.offset == 0 {
                        return Poll::Pending;
                    }
                    break;
                }
            }
        }

        let remaining = self.pending.len() - self.offset;
        if remaining == 0 {
            Poll::Ready(None)
        } else {
            let to_yield = std::cmp::min(remaining, capacity);
            let start = self.offset;
            let end = start + to_yield;
            self.offset = end;
            Poll::Ready(Some(Bytes::copy_from_slice(&self.pending[start..end])))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_chunks_bytes_single_u8() {
        let stream = stream::iter(vec![1u8, 2, 3, 4, 5]);
        let capacity = NonZeroUsize::new(2).unwrap();
        let chunks: Vec<Bytes> = stream.chunks_bytes(capacity).collect().await;

        assert_eq!(
            chunks,
            vec![
                Bytes::from_static(&[1, 2]),
                Bytes::from_static(&[3, 4]),
                Bytes::from_static(&[5]),
            ]
        );
    }

    #[tokio::test]
    async fn test_chunks_bytes_slices() {
        let stream = stream::iter(vec!["hello", " ", "world"]);
        let capacity = NonZeroUsize::new(4).unwrap();
        let chunks: Vec<Bytes> = stream.chunks_bytes(capacity).collect().await;

        assert_eq!(
            chunks,
            vec![
                Bytes::from_static(b"hell"),
                Bytes::from_static(b"o wo"),
                Bytes::from_static(b"rld"),
            ]
        );
    }

    #[tokio::test]
    async fn test_chunks_bytes_exact_boundary() {
        let stream = stream::iter(vec![10u8, 20, 30, 40]);
        let capacity = NonZeroUsize::new(2).unwrap();
        let chunks: Vec<Bytes> = stream.chunks_bytes(capacity).collect().await;

        assert_eq!(
            chunks,
            vec![Bytes::from_static(&[10, 20]), Bytes::from_static(&[30, 40]),]
        );
    }

    #[tokio::test]
    async fn test_chunks_bytes_empty() {
        let stream = stream::empty::<u8>();
        let capacity = NonZeroUsize::new(3).unwrap();
        let chunks: Vec<Bytes> = stream.chunks_bytes(capacity).collect().await;

        assert!(chunks.is_empty());
    }
}
