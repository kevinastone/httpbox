#![cfg(test)]

use crate::handler::Handler;
use crate::headers::ContentLength;
use crate::headers::{Header, HeaderMapExt};
use crate::http::{Body, Request};
use futures::prelude::*;
use hyper::Method;
use hyper::header::{HeaderName, HeaderValue};
use hyper::http::{Request as HTTPRequest, Response as HTTPResponse};
use std::convert::TryFrom;
use std::net::SocketAddr;
use uri_path::PathMatch;

pub struct RequestBuilder {
    req: HTTPRequest<Body>,
    client_addr: Option<SocketAddr>,
    params: PathMatch,
}

impl RequestBuilder {
    pub fn method(mut self, method: Method) -> Self {
        *self.req.method_mut() = method;
        self
    }

    pub fn path(mut self, p: &str) -> Self {
        let uri = p.parse().expect("test request path invalid");
        *self.req.uri_mut() = uri;
        self
    }

    pub fn param(mut self, name: &'static str, value: &str) -> Self {
        self.params.insert(name, value.to_owned());
        self
    }

    pub fn typed_header<H: Header>(mut self, header: H) -> Self {
        self.req.headers_mut().typed_insert(header);
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
    {
        let name: HeaderName = TryFrom::try_from(key)
            .map_err(|_| ())
            .expect("invalid header name");
        let value = TryFrom::try_from(value)
            .map_err(|_| ())
            .expect("invalid header value");
        self.req.headers_mut().append(name, value);
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref().to_vec();
        let len = body.len();
        *self.req.body_mut() = body.into();
        self.typed_header(ContentLength(len as u64))
    }

    pub fn client_addr(mut self, addr: SocketAddr) -> Self {
        self.client_addr = Some(addr);
        self
    }

    pub fn build(self) -> Request {
        let mut req = self.req;

        if let Some(client_addr) = self.client_addr {
            req.extensions_mut().insert(client_addr);
        }
        Request::new(req, self.params)
    }

    pub async fn handle<H: Handler>(
        self,
        handler: H,
    ) -> hyper::http::Result<HTTPResponse<Body>> {
        let req = self.build();
        handler.handle(req).or_else(|e| e.into_result()).await
    }
}

pub fn request() -> RequestBuilder {
    RequestBuilder {
        req: HTTPRequest::default(),
        client_addr: None,
        params: PathMatch::default(),
    }
}
