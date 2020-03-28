use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use hyper::http::{HeaderMap, Uri};
use hyper::Body;
use std::collections::HashMap;
use std::net::SocketAddr;

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
    client_addr: Option<SocketAddr>,
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

    pub fn params<'a, T: serde::de::Deserialize<'a>>(&self) -> Option<T> {
        de::deserialize(self.params.clone()).ok()
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

    pub fn client_addr(&self) -> Option<SocketAddr> {
        self.client_addr
    }
}
