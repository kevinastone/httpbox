use crate::headers::Host;
use crate::http::Request;
use crate::http::Uri;
use std::env;
use std::sync::LazyLock;
use url::Url;

static BASE_URL: LazyLock<Option<Url>> =
    LazyLock::new(|| env::var_os("BASE_URL")?.into_string().ok()?.parse().ok());

fn scheme_from_request(req: &Request) -> &str {
    if let Some(scheme) = req.uri().scheme_str() {
        return scheme;
    }
    if let Some(proto) = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|proto| proto.split(',').next().map(str::trim))
        .filter(|trimmed| !trimmed.is_empty())
    {
        return proto;
    }
    "http"
}

fn host_to_url(host: &str, scheme: &str) -> anyhow::Result<Url> {
    Ok(Uri::builder()
        .scheme(scheme)
        .authority(host)
        .path_and_query("/")
        .build()?
        .to_string()
        .parse::<Url>()?)
}

fn host_from_headers(req: &Request) -> anyhow::Result<Url> {
    let host = req
        .uri()
        .authority()
        .map(|auth| auth.as_str().to_string())
        .or_else(|| req.typed_header::<Host>().map(|h| h.to_string()))
        .ok_or_else(|| anyhow::anyhow!("no host or authority found"))?;

    let scheme = scheme_from_request(req);
    host_to_url(&host, scheme)
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
    use super::{
        absolute_uri, host_from_headers, host_to_url, scheme_from_request,
    };
    use crate::headers::Host;
    use crate::test::*;
    use hyper::http::{Uri, uri::Authority};

    #[test]
    fn test_host_to_url() {
        assert_eq!(
            host_to_url("example.com", "http").unwrap().to_string(),
            "http://example.com/",
        )
    }

    #[test]
    fn test_host_to_url_parse_error() {
        assert!(host_to_url("a/b/c", "http").is_err())
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

    #[test]
    fn test_host_from_uri_authority() {
        let req = request()
            .path("http://authority.example.com/some/path")
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "http://authority.example.com/"
        );
    }

    #[test]
    fn test_host_from_uri_authority_with_port() {
        let req = request()
            .path("http://authority.example.com:8080/some/path")
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "http://authority.example.com:8080/"
        );
    }

    #[test]
    fn test_scheme_from_x_forwarded_proto() {
        let req = request()
            .typed_header(Host::from(Authority::from_static("example.com")))
            .header("x-forwarded-proto", "https")
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "https://example.com/"
        );
    }

    #[test]
    fn test_scheme_from_uri() {
        let req = request()
            .path("https://secure.example.com/some/path")
            .build();

        assert_eq!(
            host_from_headers(&req).unwrap().to_string(),
            "https://secure.example.com/"
        );
    }

    #[test]
    fn test_scheme_from_request_default() {
        let req = request().build();
        assert_eq!(scheme_from_request(&req), "http");
    }

    #[test]
    fn test_scheme_from_request_uri() {
        let req = request().path("https://example.com/test").build();
        assert_eq!(scheme_from_request(&req), "https");
    }

    #[test]
    fn test_scheme_from_request_x_forwarded_proto() {
        let req = request().header("x-forwarded-proto", "https").build();
        assert_eq!(scheme_from_request(&req), "https");
    }

    #[test]
    fn test_scheme_from_request_x_forwarded_proto_chained() {
        let req = request().header("x-forwarded-proto", "https, http").build();
        assert_eq!(scheme_from_request(&req), "https");
    }

    #[test]
    fn test_scheme_from_request_x_forwarded_proto_whitespace() {
        let req = request().header("x-forwarded-proto", "   https   ").build();
        assert_eq!(scheme_from_request(&req), "https");
    }

    #[test]
    fn test_scheme_from_request_x_forwarded_proto_empty() {
        let req = request().header("x-forwarded-proto", "").build();
        assert_eq!(scheme_from_request(&req), "http");
    }

    #[test]
    fn test_scheme_from_request_uri_precedence_over_header() {
        let req = request()
            .path("https://example.com/test")
            .header("x-forwarded-proto", "http")
            .build();
        assert_eq!(scheme_from_request(&req), "https");
    }
}
