use gotham::state::{FromState, State};
use http::header;
use hyper::{HeaderMap, Uri};
use lazy_static::lazy_static;
use std::env;
use url::Url;

lazy_static! {
    static ref BASE_URL: Option<Url> = env::var_os("BASE_URL")
        .and_then(|os_str| os_str.into_string().ok())
        .and_then(|url| Url::parse(&url).ok());
}

pub fn join_url(url: &str, base: &Url) -> Option<Url> {
    base.join(url).ok()
}

fn host_to_uri(host: &str, relative_uri: &str) -> Result<Uri, String> {
    format!(
        "{}://{}",
        "http", // FIXME: Determine protocol
        host,
    )
    .parse::<Url>()
    .and_then(|url| url.join(relative_uri))
    .map_err(|e| e.to_string())
    .map(|url| url.to_string())
    .and_then(|url_str| url_str.parse::<Uri>().map_err(|e| e.to_string()))
}

fn absolute_uri(state: &State, uri: Uri) -> Result<Uri, String> {
    if uri.scheme_part().is_some() {
        Ok(uri)
    } else {
        match HeaderMap::borrow_from(&state).get(header::HOST) {
            Some(host) => {
                host_to_uri(host.to_str().unwrap(), &uri.to_string()[..])
            }
            None => Err("No host header found".into()),
        }
    }
}

pub fn absolute_url(state: &State, uri: Uri) -> Option<Url> {
    BASE_URL.clone().or_else(|| {
        absolute_uri(&state, uri)
            .ok()
            .and_then(|abs_uri| Url::parse(&abs_uri.to_string()[..]).ok())
    })
}

#[cfg(test)]
mod test {
    use super::{absolute_uri, join_url};

    use gotham::state::State;
    use http::header;
    use hyper::{HeaderMap, Uri};
    use url::Url;

    #[test]
    fn test_join_url() {
        let base = Url::parse("https://example.com").unwrap();
        assert_eq!(
            join_url("/something", &base).unwrap().to_string(),
            "https://example.com/something"
        );
    }

    #[test]
    fn test_absolute_uri() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HOST,
                header::HeaderValue::from_static("example.com"),
            );
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
            headers.insert(
                header::HOST,
                header::HeaderValue::from_static("example.com:1234"),
            );
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
