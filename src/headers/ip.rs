use crate::headers::{Error, Header, HeaderName, HeaderValue};
use lazy_static::lazy_static;
use std::iter;
use std::net::IpAddr;

lazy_static! {
    static ref X_FORWARDED_FOR: HeaderName =
        HeaderName::from_static("x-forwarded-for");
}

#[derive(Clone, Debug, PartialEq)]
pub struct XForwardedFor(pub IpAddr);

impl XForwardedFor {
    pub fn ip_addr(&self) -> IpAddr {
        self.0
    }
}

impl Header for XForwardedFor {
    fn name() -> &'static HeaderName {
        &X_FORWARDED_FOR
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .and_then(|v| v.to_str().ok()?.parse().ok())
            .map(XForwardedFor)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(iter::once(self.into()))
    }
}

impl From<&XForwardedFor> for HeaderValue {
    fn from(x_forwarded_for: &XForwardedFor) -> Self {
        x_forwarded_for.0.to_string().parse().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::XForwardedFor;
    use crate::headers::{Header, HeaderMapExt};
    use crate::test::headers::encode;
    use hyper::http::HeaderMap;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_encode_ipv4() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(
            encode(XForwardedFor(ip_addr)).to_str().unwrap(),
            "127.0.0.1"
        )
    }

    #[test]
    fn test_encode_ipv6() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(encode(XForwardedFor(ip_addr)).to_str().unwrap(), "::1")
    }

    #[test]
    fn test_decode_ipv4() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let mut headers = HeaderMap::new();
        headers.insert(XForwardedFor::name(), "127.0.0.1".parse().unwrap());

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location, XForwardedFor(ip_addr))
    }

    #[test]
    fn test_decode_ipv6() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let mut headers = HeaderMap::new();
        headers.insert(XForwardedFor::name(), "::1".parse().unwrap());

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location, XForwardedFor(ip_addr))
    }
}
