use super::Body;
use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use std::net::SocketAddr;
use uri_path::PathMatch;

pub struct Request {
    req: HTTPRequest<Body>,
    params: PathMatch,
}

impl Request {
    pub fn new(req: HTTPRequest<Body>, params: PathMatch) -> Self {
        Self { req, params }
    }

    pub fn param<T: std::str::FromStr>(&self, key: &'static str) -> Option<T> {
        let str = self.params.get(key)?;
        T::from_str(str).ok()
    }

    pub fn params<'a, T: serde::de::Deserialize<'a>>(&'a self) -> Option<T> {
        self.params.deserialize().ok()
    }

    pub fn typed_header<H: Header>(&self) -> Option<H> {
        self.req.headers().typed_get::<H>()
    }

    pub fn query<'a, T: serde::de::Deserialize<'a>>(
        &'a self,
    ) -> std::result::Result<T, serde_urlencoded::de::Error> {
        let query_string = self.req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query_string)
    }

    pub fn client_addr(&self) -> Option<&SocketAddr> {
        self.req.extensions().get::<SocketAddr>()
    }
}

impl core::ops::Deref for Request {
    type Target = HTTPRequest<Body>;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl core::ops::DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.req
    }
}
