use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use hyper::http::{HeaderMap, Uri};
use hyper::Body;
use std::net::SocketAddr;

pub struct Request {
    req: HTTPRequest<Body>,
    client_addr: Option<SocketAddr>,
}

impl Request {
    pub fn new(
        req: HTTPRequest<Body>,
        client_addr: Option<SocketAddr>,
    ) -> Self {
        Self { req, client_addr }
    }

    pub fn req(&self) -> &HTTPRequest<Body> {
        &self.req
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

    pub fn query<'a, TQ: serde::de::Deserialize<'a>>(
        &'a self,
    ) -> std::result::Result<TQ, serde_urlencoded::de::Error> {
        let query_string = self.req.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query_string)
    }

    pub fn client_addr(&self) -> Option<SocketAddr> {
        self.client_addr
    }
}

// impl<'a, T: serde::de::Deserialize<'a>> Request<T> {
//     pub fn params(&self) -> Option<T> {
//         self.path.parse(self.req.path())
//     }
// }
