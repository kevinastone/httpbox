use crate::headers::{Error, Header, HeaderName, HeaderValue};
use cookie::Cookie as HTTPCookie;
use hyper::http::header;
use itertools::Either;
use std::iter;

static COOKIE: &HeaderName = &header::COOKIE;
static SET_COOKIE: &HeaderName = &header::SET_COOKIE;

#[derive(Clone, Debug, PartialEq)]
pub struct Cookie<'a>(pub Vec<HTTPCookie<'a>>);

impl Cookie<'_> {
    pub fn iter(&self) -> impl Iterator<Item = &HTTPCookie<'_>> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for Cookie<'a> {
    type Item = HTTPCookie<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Header for Cookie<'_> {
    fn name() -> &'static HeaderName {
        COOKIE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .map(|h| h.to_str().map_err(|_| Error::invalid()))
            .flat_map(|h| match h {
                Ok(v) => Either::Left(v.split(';').map(|s| Ok(s.trim()))),
                Err(e) => Either::Right(iter::once(Err(e))),
            })
            .map(|item| {
                item.and_then(|s| {
                    HTTPCookie::parse(s)
                        .map_err(|_| Error::invalid())
                        .map(|c| c.into_owned())
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Cookie)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let cookies = self.0.iter().map(|c| c.to_string()).collect::<Vec<_>>();

        values.extend(iter::once(cookies.join("; ").parse().unwrap()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetCookie<'a>(pub HTTPCookie<'a>);

impl Header for SetCookie<'_> {
    fn name() -> &'static HeaderName {
        SET_COOKIE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .and_then(|v| HTTPCookie::parse(v.to_str().ok()?.to_string()).ok())
            .map(SetCookie)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(iter::once(self.0.to_string().parse().unwrap()))
    }
}

#[cfg(test)]
mod test {
    use super::{Cookie, HTTPCookie, SetCookie};
    use crate::headers::{Header, HeaderMapExt, HeaderValue};
    use crate::test::headers::encode;
    use hyper::http::HeaderMap;

    #[test]
    fn test_encode_single_cookie() {
        let cookie = HTTPCookie::new("name", "value");
        assert_eq!(encode(Cookie(vec![cookie])).to_str().unwrap(), "name=value")
    }

    #[test]
    fn test_encode_multiple_cookies() {
        assert_eq!(
            encode(Cookie(vec![
                HTTPCookie::new("first", "value"),
                HTTPCookie::new("second", "another")
            ]))
            .to_str()
            .unwrap(),
            "first=value; second=another"
        )
    }

    #[test]
    fn test_decode_single_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(Cookie::name(), "name=value".parse().unwrap());

        let header = headers.typed_get::<Cookie>().unwrap();
        assert_eq!(header, Cookie(vec![HTTPCookie::new("name", "value")]))
    }

    #[test]
    fn test_decode_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            Cookie::name(),
            "first=value; second=another".parse().unwrap(),
        );

        let header = headers.typed_get::<Cookie>().unwrap();
        assert_eq!(
            header,
            Cookie(vec![
                HTTPCookie::new("first", "value"),
                HTTPCookie::new("second", "another")
            ])
        )
    }

    #[test]
    fn test_decode_cookie_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(Cookie::name(), "abc".parse().unwrap());

        let header = headers.typed_try_get::<Cookie>();
        assert!(header.is_err())
    }

    #[test]
    fn test_decode_cookie_invalid_not_str() {
        let mut headers = HeaderMap::new();
        headers
            .insert(Cookie::name(), HeaderValue::from_bytes(b"\xfa").unwrap());

        let header = headers.typed_try_get::<Cookie>();
        assert!(header.is_err())
    }

    #[test]
    fn test_encode_set_cookie() {
        let cookie = HTTPCookie::new("name", "value");
        assert_eq!(encode(SetCookie(cookie)).to_str().unwrap(), "name=value")
    }

    #[test]
    fn test_decode_set_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(SetCookie::name(), "name=value".parse().unwrap());

        let header = headers.typed_get::<SetCookie>().unwrap();
        assert_eq!(header, SetCookie(HTTPCookie::new("name", "value")))
    }

    #[test]
    fn test_decode_set_cookie_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(SetCookie::name(), "abc".parse().unwrap());

        let header = headers.typed_try_get::<SetCookie>();
        assert!(header.is_err())
    }
}
