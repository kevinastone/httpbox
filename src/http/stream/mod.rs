use crate::http::Body;
pub(crate) use chunk_bytes::*;
use futures::prelude::*;
use hyper::body::Bytes;

pub(crate) fn ok_stream<T, S: Stream<Item = T>>(
    stream: S,
) -> impl Stream<Item = Result<T, !>> {
    stream.map(Ok)
}

pub(crate) fn body_from_stream<S: Stream + Send + Sync + 'static>(
    stream: S,
) -> Body
where
    S::Item: Into<Bytes>,
{
    Body::from_stream(ok_stream(stream))
}
