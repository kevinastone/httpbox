use crate::headers::{Error, Header, HeaderName, HeaderValue};
use lazy_static::lazy_static;
use std::iter;
use std::net::IpAddr;

lazy_static! {
    static ref X_FORWARDED_FOR: HeaderName =
        HeaderName::from_static("x-forwarded-for");
}

#[derive(Clone, Debug, PartialEq)]
pub struct XForwardedFor {
    pub client: IpAddr,
    pub proxies: Vec<IpAddr>,
}

impl XForwardedFor {
    pub fn client(client: IpAddr) -> Self {
        Self {
            client,
            proxies: vec![],
        }
    }

    pub fn client_with_proxies(client: IpAddr, proxies: Vec<IpAddr>) -> Self {
        Self { client, proxies }
    }

    pub fn ip_addr(&self) -> IpAddr {
        self.client
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
        let mut steps = values.flat_map(|value| {
            value.to_str().into_iter().flat_map(|string| {
                string
                    .split(',')
                    .filter_map(|x| match x.trim() {
                        "" => None,
                        y => Some(y),
                    })
                    .map(|x| x.parse().map_err(|_| Error::invalid()))
            })
        });

        let client = steps.next().ok_or_else(Error::invalid)??;
        let proxies = steps.collect::<Result<_, _>>()?;

        Ok(XForwardedFor { client, proxies })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(iter::once(self.into()))
    }
}

impl From<&XForwardedFor> for HeaderValue {
    fn from(x_forwarded_for: &XForwardedFor) -> Self {
        let mut output = x_forwarded_for.client.to_string();
        for proxy in &x_forwarded_for.proxies {
            output += ", ";
            output += &proxy.to_string();
        }

        output.parse().unwrap()
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
            encode(XForwardedFor::client(ip_addr)).to_str().unwrap(),
            "127.0.0.1"
        )
    }

    #[test]
    fn test_encode_ipv4_with_proxies() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let proxy = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        assert_eq!(
            encode(XForwardedFor::client_with_proxies(ip_addr, vec![proxy]))
                .to_str()
                .unwrap(),
            "127.0.0.1, 127.0.0.2"
        )
    }

    #[test]
    fn test_encode_ipv6() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(
            encode(XForwardedFor::client(ip_addr)).to_str().unwrap(),
            "::1"
        )
    }

    #[test]
    fn test_encode_ipv6_with_proxies() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let proxy = IpAddr::V6(Ipv6Addr::new(10, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(
            encode(XForwardedFor::client_with_proxies(ip_addr, vec![proxy]))
                .to_str()
                .unwrap(),
            "::1, a::1"
        )
    }

    #[test]
    fn test_decode_ipv4() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let mut headers = HeaderMap::new();
        headers.insert(XForwardedFor::name(), "127.0.0.1".parse().unwrap());

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location, XForwardedFor::client(ip_addr))
    }

    #[test]
    fn test_decode_ipv4_with_proxies() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let proxy = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        let mut headers = HeaderMap::new();
        headers.insert(
            XForwardedFor::name(),
            "127.0.0.1, 127.0.0.2".parse().unwrap(),
        );

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location.client, ip_addr);
        assert_eq!(location.proxies.first(), Some(&proxy));
    }

    #[test]
    fn test_decode_ipv6() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let mut headers = HeaderMap::new();
        headers.insert(XForwardedFor::name(), "::1".parse().unwrap());

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location, XForwardedFor::client(ip_addr))
    }

    #[test]
    fn test_decode_ipv6_with_proxies() {
        let ip_addr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let proxy = IpAddr::V6(Ipv6Addr::new(10, 0, 0, 0, 0, 0, 0, 1));
        let mut headers = HeaderMap::new();
        headers.insert(XForwardedFor::name(), "::1, a::1".parse().unwrap());

        let location = headers.typed_get::<XForwardedFor>().unwrap();
        assert_eq!(location.client, ip_addr);
        assert_eq!(location.proxies.first(), Some(&proxy));
    }
}
