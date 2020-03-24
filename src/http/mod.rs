use crate::headers::{Header, HeaderMapExt};
pub use hyper::http::{
    HeaderMap, Request as HTTPRequest, Response as HTTPResponse, StatusCode,
    Uri,
};
use std::collections::HashMap;

pub use hyper::{body::Bytes, Body};
use std::net::SocketAddr;

mod header;
mod response;
mod stream;

pub use header::*;
pub use response::*;
pub(crate) use stream::*;

pub type Response = HTTPResponse<Body>;

#[derive(Debug)]
pub enum HandlerError {
    HyperError(hyper::http::Error),
    Failure(Response),
}

impl HandlerError {
    pub fn into_result(self) -> hyper::http::Result<Response> {
        match self {
            Self::HyperError(e) => Err(e),
            Self::Failure(res) => Ok(res),
        }
    }
}

impl From<hyper::http::Result<Response>> for HandlerError {
    fn from(result: hyper::http::Result<Response>) -> Self {
        match result {
            Ok(res) => Self::Failure(res),
            Err(e) => Self::HyperError(e),
        }
    }
}

impl From<hyper::http::Error> for HandlerError {
    fn from(error: hyper::http::Error) -> Self {
        HandlerError::HyperError(error)
    }
}

pub type Result = std::result::Result<Response, HandlerError>;

pub struct Request {
    pub req: HTTPRequest<Body>,
    pub client_addr: Option<SocketAddr>,
    params: HashMap<&'static str, String>,
}

impl Request {
    pub fn new(
        req: HTTPRequest<Body>,
        client_addr: Option<SocketAddr>,
        params: Option<HashMap<&'static str, String>>,
    ) -> Self {
        Self {
            req,
            client_addr,
            params: params.unwrap_or_else(HashMap::new),
        }
    }

    pub fn param<T: std::str::FromStr>(&self, key: &'static str) -> Option<T> {
        let str = self.params.get(key)?;
        T::from_str(str).ok()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.req.headers()
    }

    pub fn uri(&self) -> &Uri {
        self.req.uri()
    }

    pub fn body(&mut self) -> Body {
        std::mem::replace(self.req.body_mut(), Body::empty())
    }

    pub fn typed_header<H: Header>(&self) -> Option<H> {
        self.req.headers().typed_get::<H>()
    }

    pub fn query<T: serde::de::DeserializeOwned>(
        &self,
    ) -> std::result::Result<T, serde_urlencoded::de::Error> {
        let query_string = self.req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query_string)
    }
}
