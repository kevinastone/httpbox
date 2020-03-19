use futures::prelude::*;
use hyper::body::Bytes;
use hyper::Body;
use std::convert::Infallible;

pub(crate) fn ok_stream<T, S: Stream<Item = T>>(
    stream: S,
) -> impl TryStream<Ok = T, Error = Infallible> {
    stream.map(Ok)
}

pub(crate) fn body_from_stream<S: Stream + Send + Sync + 'static + Unpin>(
    stream: S,
) -> Body
where
    Bytes: From<<S as Stream>::Item>,
{
    Body::wrap_stream(ok_stream(stream).into_stream())
}
