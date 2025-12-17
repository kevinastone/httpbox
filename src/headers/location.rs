use crate::headers::{Error, Header, HeaderName, HeaderValue};
use hyper::http::{Uri, header};
use std::iter;

static LOCATION: &HeaderName = &header::LOCATION;

#[derive(Clone, Debug, PartialEq)]
pub struct Location(Uri);

impl Location {
    pub fn uri(&self) -> &Uri {
        &self.0
    }
}

impl Header for Location {
    fn name() -> &'static HeaderName {
        LOCATION
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .and_then(|v| v.to_str().ok()?.parse().ok())
            .map(Location)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(iter::once(self.into()))
    }
}

impl From<Uri> for Location {
    fn from(uri: Uri) -> Self {
        Location(uri)
    }
}

impl From<&Location> for HeaderValue {
    fn from(location: &Location) -> Self {
        location.0.to_string().parse().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::Location;
    use crate::headers::HeaderMapExt;
    use crate::test::headers::encode;
    use hyper::http::{HeaderMap, Uri, header};

    #[test]
    fn test_encode_relative_location() {
        let uri: Uri = "/foo/bar?baz".parse().unwrap();
        assert_eq!(encode(Location(uri)).to_str().unwrap(), "/foo/bar?baz")
    }

    #[test]
    fn test_encode_absolute_location() {
        let uri: Uri = "https://example.com/foo/bar?baz".parse().unwrap();
        assert_eq!(
            encode(Location(uri)).to_str().unwrap(),
            "https://example.com/foo/bar?baz"
        )
    }

    #[test]
    fn test_decode_relative_location() {
        let uri = "/foo/bar?baz";

        let mut headers = HeaderMap::new();
        headers.insert(header::LOCATION, uri.parse().unwrap());

        let location = headers.typed_get::<Location>().unwrap();
        assert_eq!(location.uri(), &(uri.parse::<Uri>().unwrap()))
    }

    #[test]
    fn test_decode_absolute_location() {
        let uri = "https://example.com/foo/bar?baz";

        let mut headers = HeaderMap::new();
        headers.insert(header::LOCATION, uri.parse().unwrap());

        let location = headers.typed_get::<Location>().unwrap();
        assert_eq!(location.uri(), &(uri.parse::<Uri>().unwrap()))
    }
}
