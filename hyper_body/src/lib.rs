use futures::prelude::*;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::{Body as HttpBody, Bytes, Frame, Incoming, SizeHint};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{error::Error as StdError, fmt};

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, Error>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Error {
    inner: BoxError,
}

impl Error {
    /// Create a new `Error` from a boxable error.
    pub fn new(error: impl Into<BoxError>) -> Self {
        Self {
            inner: error.into(),
        }
    }

    /// Convert an `Error` back into the underlying boxed trait object.
    pub fn into_inner(self) -> BoxError {
        self.inner
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.inner)
    }
}

fn boxed<B>(body: B) -> BoxBody
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    try_downcast(body)
        .unwrap_or_else(|body| body.map_err(Error::new).boxed_unsync())
}

pub(crate) fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

#[derive(Debug)]
pub struct Body(BoxBody);

impl Body {
    /// Create a new `Body` that wraps another [`HttpBody`].
    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
    {
        Self(boxed(body))
    }

    /// Create an empty body.
    pub fn empty() -> Self {
        Self::new(http_body_util::Empty::new())
    }

    /// Create a new `Body` from a [`Stream`].
    ///
    /// [`Stream`]: futures::stream::Stream
    pub fn from_stream<S, D, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<D, E>> + Sync + Send + 'static,
        D: Into<Bytes>,
        E: Into<BoxError>,
    {
        Self::new(StreamBody::new(
            stream.map_ok(|item| Frame::data(item.into())),
        ))
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Incoming> for Body {
    fn from(incoming: Incoming) -> Self {
        Self::new(incoming)
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(http_body_util::Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(std::borrow::Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(std::borrow::Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl HttpBody for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.0).poll_frame(cx)
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        self.0.size_hint()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_downcast() {
        assert_eq!(try_downcast::<i32, _>(5_u32), Err(5_u32));
        assert_eq!(try_downcast::<i32, _>(5_i32), Ok(5_i32));
    }
}
