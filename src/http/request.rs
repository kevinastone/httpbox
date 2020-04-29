use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use hyper::http::{HeaderMap, Uri};
use hyper::Body;
use std::net::SocketAddr;
use typed_path::Path;

mod de {
    use serde::de::{value::Error, Deserialize, IntoDeserializer};

    pub fn deserialize<'de, IS, T>(raw: IS) -> Result<T, Error>
    where
        IS: IntoDeserializer<'de, Error>,
        T: Deserialize<'de>,
    {
        let deserializer = raw.into_deserializer();
        T::deserialize(deserializer)
    }
}

pub struct Request<T = ()> {
    req: HTTPRequest<Body>,
    client_addr: Option<SocketAddr>,
    path: Path<T>,
}

impl<T> Request<T> {
    pub fn new(
        req: HTTPRequest<Body>,
        client_addr: Option<SocketAddr>,
        path: Path<T>,
    ) -> Self {
        Self {
            req,
            client_addr,
            path,
        }
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

impl<'a, T: serde::de::Deserialize<'a>> Request<T> {
    pub fn params(&self) -> Option<T> {
        self.path.parse(self.req.path())
    }
}
