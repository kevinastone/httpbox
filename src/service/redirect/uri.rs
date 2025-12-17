use crate::headers::Host;
use crate::http::Request;
use crate::http::Uri;
use std::env;
use std::sync::LazyLock;
use url::Url;

static BASE_URL: LazyLock<Option<Url>> =
    LazyLock::new(|| env::var_os("BASE_URL")?.into_string().ok()?.parse().ok());

fn host_to_url(host: &str) -> anyhow::Result<Url> {
    Ok(Uri::builder()
        .scheme("http") // FIXME: Determine protocol
        .authority(host)
        .path_and_query("/")
        .build()?
        .to_string()
        .parse::<Url>()?)
}

fn host_from_headers(req: &Request) -> anyhow::Result<Url> {
    let host = req
        .typed_header::<Host>()
        .ok_or_else(|| anyhow::anyhow!("no host header found"))?
        .to_string();

    host_to_url(&host)
}

fn absolute_uri(req: &Request, uri: &Uri) -> anyhow::Result<Uri> {
    if uri.scheme().is_some() {
        Ok(uri.clone())
    } else {
        let base = match BASE_URL.clone() {
            Some(url) => url,
            None => host_from_headers(req)?,
        };

        Ok(base.join(&uri.to_string())?.to_string().parse()?)
    }
}

pub fn absolute_url(req: &Request, uri: &Uri) -> anyhow::Result<Url> {
    Ok(absolute_uri(req, uri)?.to_string().parse()?)
}

#[cfg(test)]
mod test {
    use super::{absolute_uri, host_from_headers, host_to_url};
    use crate::headers::Host;
    use crate::test::*;
    use hyper::http::{Uri, uri::Authority};

    #[test]
    fn test_host_to_url() {
        assert_eq!(
            host_to_url("example.com").unwrap().to_string(),
            "http://example.com/",
        )
    }

    #[test]
    fn test_host_to_url_parse_error() {
        assert!(host_to_url("a/b/c").is_err())
    }

    #[test]
    fn test_host_from_headers() {
        let req = request()
            .typed_header(Host::from(Authority::from_static("example.com")))
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "http://example.com/"
        )
    }

    #[test]
    fn test_host_from_headers_with_port() {
        let req = request()
            .typed_header(Host::from(Authority::from_static(
                "example.com:1234",
            )))
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "http://example.com:1234/"
        )
    }

    #[test]
    fn test_host_from_headers_no_header() {
        let req = request().build();

        assert!(host_from_headers(&req).is_err())
    }

    #[test]
    fn test_absolute_uri() {
        let req = request()
            .typed_header(Host::from(Authority::from_static("example.com")))
            .build();

        let relative_uri = "/first/second?a=1&b=2".parse::<Uri>().unwrap();
        let absolute_uri = absolute_uri(&req, &relative_uri).unwrap();
        assert_eq!(
            absolute_uri.to_string(),
            "http://example.com/first/second?a=1&b=2"
        )
    }

    #[test]
    fn test_absolute_uri_with_port() {
        let req = request()
            .typed_header(Host::from(Authority::from_static(
                "example.com:1234",
            )))
            .build();

        let relative_uri = "/first/second".parse::<Uri>().unwrap();
        let absolute_uri = absolute_uri(&req, &relative_uri).unwrap();
        assert_eq!(
            absolute_uri.to_string(),
            "http://example.com:1234/first/second"
        )
    }
}
