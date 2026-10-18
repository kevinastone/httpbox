use super::Body;
use crate::headers::{Header, HeaderMapExt};
use hyper::http::Request as HTTPRequest;
use std::net::SocketAddr;
use uri_path::PathMatch;

pub type Request = HTTPRequest<Body>;

pub trait RequestExt {
    fn param<T: std::str::FromStr>(&self, key: &'static str) -> Option<T>;

    fn params<'a, T: serde::de::Deserialize<'a>>(&'a self) -> Option<T>;

    fn typed_header<H: Header>(&self) -> Option<H>;

    fn query<'a, T: serde::de::Deserialize<'a>>(
        &'a self,
    ) -> std::result::Result<T, serde_urlencoded::de::Error>;

    fn client_addr(&self) -> Option<&SocketAddr>;
}

impl<B> RequestExt for HTTPRequest<B> {
    fn param<T: std::str::FromStr>(&self, key: &'static str) -> Option<T> {
        let params = self.extensions().get::<PathMatch>()?;
        let str = params.get(key)?;
        T::from_str(str).ok()
    }

    fn params<'a, T: serde::de::Deserialize<'a>>(&'a self) -> Option<T> {
        let params = self.extensions().get::<PathMatch>()?;
        params.deserialize().ok()
    }

    fn typed_header<H: Header>(&self) -> Option<H> {
        self.headers().typed_get::<H>()
    }

    fn query<'a, T: serde::de::Deserialize<'a>>(
        &'a self,
    ) -> std::result::Result<T, serde_urlencoded::de::Error> {
        let query_string = self.uri().query().unwrap_or("");
        serde_urlencoded::from_str(query_string)
    }

    fn client_addr(&self) -> Option<&SocketAddr> {
        self.extensions().get::<SocketAddr>()
    }
}
