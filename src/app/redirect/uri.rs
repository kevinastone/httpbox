use failure::{format_err, Fallible};
use gotham::state::{FromState, State};
use http::header;
use hyper::{HeaderMap, Uri};
use lazy_static::lazy_static;
use std::env;
use url::Url;

lazy_static! {
    static ref BASE_URL: Option<Url> =
        env::var_os("BASE_URL")?.into_string().ok()?.parse().ok();
}

fn host_to_url(host: &str) -> Fallible<Url> {
    Ok(Uri::builder()
        .scheme("http") // FIXME: Determine protocol
        .authority(host)
        .path_and_query("/")
        .build()?
        .to_string()
        .parse::<Url>()?)
}

fn host_from_headers(state: &State) -> Fallible<Url> {
    Ok(HeaderMap::borrow_from(&state)
        .get(header::HOST)
        .ok_or_else(|| format_err!("No host header found"))
        .and_then(|host| Ok(host.to_str()?))
        .and_then(|host| host_to_url(host))?)
}

fn absolute_uri(state: &State, uri: Uri) -> Fallible<Uri> {
    if uri.scheme_part().is_some() {
        Ok(uri)
    } else {
        let base = match BASE_URL.clone() {
            Some(url) => url,
            None => host_from_headers(&state)?,
        };

        Ok(base.join(&uri.to_string())?.to_string().parse()?)
    }
}

pub fn absolute_url(state: &State, uri: Uri) -> Fallible<Url> {
    Ok(absolute_uri(&state, uri)?.to_string().parse()?)
}

#[cfg(test)]
mod test {
    use super::{absolute_uri, host_from_headers, host_to_url};

    use gotham::state::State;
    use headers_ext::{HeaderMapExt, Host};
    use http::uri::Authority;
    use hyper::{HeaderMap, Uri};

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
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(Host::from(Authority::from_static(
                "example.com",
            )));
            state.put(headers);

            assert_eq!(
                host_from_headers(&state).unwrap().to_string(),
                "http://example.com/"
            )
        })
    }

    #[test]
    fn test_host_from_headers_with_port() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(Host::from(Authority::from_static(
                "example.com:1234",
            )));
            state.put(headers);

            assert_eq!(
                host_from_headers(&state).unwrap().to_string(),
                "http://example.com:1234/"
            )
        })
    }

    #[test]
    fn test_host_from_headers_no_header() {
        State::with_new(|state| {
            state.put(HeaderMap::new());

            assert!(host_from_headers(&state).is_err())
        })
    }

    #[test]
    fn test_absolute_uri() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(Host::from(Authority::from_static(
                "example.com",
            )));
            state.put(headers);

            let relative_uri = "/first/second?a=1&b=2".parse::<Uri>().unwrap();
            let absolute_uri = absolute_uri(&state, relative_uri).unwrap();
            assert_eq!(
                absolute_uri.to_string(),
                "http://example.com/first/second?a=1&b=2"
            )
        })
    }

    #[test]
    fn test_absolute_uri_with_port() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(Host::from(Authority::from_static(
                "example.com:1234",
            )));
            state.put(headers);

            let relative_uri = "/first/second".parse::<Uri>().unwrap();
            let absolute_uri = absolute_uri(&state, relative_uri).unwrap();
            assert_eq!(
                absolute_uri.to_string(),
                "http://example.com:1234/first/second"
            )
        })
    }
}
