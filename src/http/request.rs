use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use hyper::Body;
use std::net::SocketAddr;
use uri_path::PathMatch;

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

    pub fn params<'a, T: serde::de::Deserialize<'a>>(&self) -> Option<T> {
        de::deserialize(self.params.clone()).ok()
    }

    pub fn body(&mut self) -> Body {
        std::mem::replace(self.req.body_mut(), Body::empty())
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
