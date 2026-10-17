use bytes::{Bytes, BytesMut};
use futures::prelude::*;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsByteSlice {
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

pub trait StreamChunkBytesExt: Stream
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
            pending: BytesMut::new(),
        }
    }
}

impl<S: Stream + ?Sized> StreamChunkBytesExt for S where S::Item: AsByteSlice {}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct ChunksBytes<S> {
    stream: S,
    capacity: NonZeroUsize,
    pending: BytesMut,
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

        if self.pending.len() >= capacity {
            return Poll::Ready(Some(self.pending.split_to(capacity).freeze()));
        }

        while self.pending.len() < capacity {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.pending.extend_from_slice(chunk.as_byte_slice());
                }
                Poll::Ready(None) => break,
                Poll::Pending => {
                    if self.pending.is_empty() {
                        return Poll::Pending;
                    }
                    break;
                }
            }
        }

        if self.pending.is_empty() {
            Poll::Ready(None)
        } else {
            let to_yield = std::cmp::min(self.pending.len(), capacity);
            Poll::Ready(Some(self.pending.split_to(to_yield).freeze()))
        }
    }
}

impl<S> stream::FusedStream for ChunksBytes<S>
where
    S: stream::FusedStream + Unpin,
    S::Item: AsByteSlice,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.pending.is_empty()
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

    #[tokio::test]
    async fn test_chunks_bytes_fused_stream() {
        use futures::stream::FusedStream;

        let stream = stream::iter(vec![1u8, 2, 3]).fuse();
        let capacity = NonZeroUsize::new(2).unwrap();
        let mut chunk_stream = stream.chunks_bytes(capacity);

        assert!(!chunk_stream.is_terminated());
        let chunk1 = chunk_stream.next().await;
        assert_eq!(chunk1, Some(Bytes::from_static(&[1, 2])));
        assert!(!chunk_stream.is_terminated());

        let chunk2 = chunk_stream.next().await;
        assert_eq!(chunk2, Some(Bytes::from_static(&[3])));
        assert!(chunk_stream.is_terminated());

        let chunk3 = chunk_stream.next().await;
        assert_eq!(chunk3, None);
        assert!(chunk_stream.is_terminated());
    }

    proptest::proptest! {
        #[test]
        fn test_chunks_bytes_conservation(
            input_slices in proptest::collection::vec(proptest::collection::vec(proptest::num::u8::ANY, 0..512), 0..20),
            capacity in 1usize..2048,
        ) {
            let flat_expected: Vec<u8> = input_slices.iter().flatten().copied().collect();
            let cap = NonZeroUsize::new(capacity).unwrap();

            let stream = stream::iter(input_slices);
            let chunks: Vec<Bytes> = futures::executor::block_on(stream.chunks_bytes(cap).collect());

            let flat_actual: Vec<u8> =
                chunks.iter().flat_map(|b| b.as_ref()).copied().collect();
            proptest::prop_assert_eq!(flat_actual, flat_expected);
        }

        #[test]
        fn test_chunks_bytes_size_bounds(
            input_slices in proptest::collection::vec(proptest::collection::vec(proptest::num::u8::ANY, 0..256), 0..10),
            capacity in 1usize..512,
        ) {
            let cap = NonZeroUsize::new(capacity).unwrap();
            let stream = stream::iter(input_slices);
            let chunks: Vec<Bytes> = futures::executor::block_on(stream.chunks_bytes(cap).collect());

            for (i, chunk) in chunks.iter().enumerate() {
                proptest::prop_assert!(!chunk.is_empty(), "chunk {} was empty", i);
                proptest::prop_assert!(
                    chunk.len() <= capacity,
                    "chunk {} exceeded capacity (len: {}, cap: {})",
                    i,
                    chunk.len(),
                    capacity
                );
            }
        }
    }
}
