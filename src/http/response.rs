use super::{Body, HTTPResponse, HandlerError, Result, StatusCode, Uri};
use crate::headers::{ContentType, Header, HeaderMapExt, Location};

pub trait ResponseTypedHeaderExt {
    fn typed_header<H: Header>(self, header: H) -> Self;
}

impl ResponseTypedHeaderExt for hyper::http::response::Builder {
    fn typed_header<H: Header>(mut self, header: H) -> Self {
        self.headers_mut().unwrap().typed_insert(header);
        self
    }
}

mod wrapper {
    use super::{Body, HandlerError, ResponseTypedHeaderExt, Result};
    use crate::headers::Header;
    use hyper::header::{HeaderName, HeaderValue};
    use hyper::StatusCode;
    use std::convert::TryFrom;

    pub struct ResponseWrapper(pub hyper::http::response::Builder);

    impl ResponseWrapper {
        pub fn status(mut self, status: StatusCode) -> Self {
            self.0 = self.0.status(status);
            self
        }

        pub fn typed_header<H: Header>(mut self, header: H) -> Self {
            self.0 = self.0.typed_header(header);
            self
        }

        pub fn header<K, V>(mut self, key: K, value: V) -> Self
        where
            HeaderName: TryFrom<K>,
            <HeaderName as TryFrom<K>>::Error: Into<hyper::http::Error>,
            HeaderValue: TryFrom<V>,
            <HeaderValue as TryFrom<V>>::Error: Into<hyper::http::Error>,
        {
            self.0 = self.0.header(key, value);
            self
        }

        pub fn body<B: Into<Body>>(self, body: B) -> Result {
            self.0.body(body.into()).map_err(Into::into)
        }
    }

    impl From<ResponseWrapper> for Result {
        fn from(response: ResponseWrapper) -> Self {
            response.0.body(Body::empty()).map_err(Into::into)
        }
    }

    impl From<ResponseWrapper> for HandlerError {
        fn from(response: ResponseWrapper) -> Self {
            response.0.body(Body::empty()).into()
        }
    }
}

pub fn response() -> self::wrapper::ResponseWrapper {
    self::wrapper::ResponseWrapper(HTTPResponse::builder())
}

pub fn html<B: Into<Body>>(body: B) -> Result {
    response().typed_header(ContentType::html()).body(body)
}

pub fn ok<B>(body: B) -> Result
where
    B: Into<Body>,
{
    response().typed_header(ContentType::text()).body(body)
}

pub fn not_found() -> HandlerError {
    response().status(StatusCode::NOT_FOUND).into()
}

pub fn bad_request() -> HandlerError {
    response().status(StatusCode::BAD_REQUEST).into()
}

pub fn redirect_to(uri: Uri) -> Result {
    response()
        .status(StatusCode::FOUND)
        .typed_header(Location::from(uri))
        .into()
}
