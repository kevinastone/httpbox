use crate::http::Body;
use futures::prelude::*;
use hyper::body::Bytes;
use std::convert::Infallible;

pub(crate) fn ok_stream<T, S: Stream<Item = T>>(
    stream: S,
) -> impl Stream<Item = Result<T, Infallible>> {
    stream.map(Ok)
}

pub(crate) fn body_from_stream<S: Stream + Send + Sync + 'static + Unpin>(
    stream: S,
) -> Body
where
    S::Item: Into<Bytes>,
{
    Body::from_stream(ok_stream(stream))
}
