use futures::compat::Compat;
use futures::prelude::*;
pub use http::{HeaderMap, Response as HTTPResponse, StatusCode, Uri};
pub use hyper::{Body, Chunk};
use std::convert::Infallible;

pub type Response = HTTPResponse<Body>;

pub(crate) fn ok_stream<T, S: Stream<Item = T>>(
    stream: S,
) -> impl TryStream<Ok = T, Error = Infallible> {
    stream.map(Ok)
}

pub(crate) fn body_from_stream<S: Stream + Send + 'static + Unpin>(
    stream: S,
) -> Body
where
    Chunk: From<<S as Stream>::Item>,
{
    Body::wrap_stream(Compat::new(ok_stream(stream)))
}
