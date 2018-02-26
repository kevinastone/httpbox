extern crate gotham;
extern crate hyper;
extern crate mime;

use gotham::state::{FromState, State};
use hyper::{header, Headers, Uri};
use std::env;
use url::Url;

lazy_static! {
    static ref BASE_URL: Option<Url> =
        env::var_os("BASE_URL")
            .and_then(|os_str| os_str.into_string().ok())
            .and_then(|url| Url::parse(&url).ok());
}

pub fn join_url<'a, 'b>(url: &'b str, base: &'a Url) -> Option<Url> {
    base.join(url).ok()
}

fn host_to_uri(host: &header::Host, relative_uri: &str) -> Result<Uri, String> {
    format!(
        "{}://{}{}",
        "http", // FIXME: Determine protocol
        host.hostname(),
        host.port()
            .map(|p| format!(":{}", p))
            .unwrap_or(String::from("")),
    ).parse::<Url>()
        .and_then(|url| url.join(relative_uri))
        .map_err(|e| e.to_string())
        .map(|url| url.to_string())
        .and_then(|url_str| url_str.parse::<Uri>().map_err(|e| e.to_string()))
}

fn absolute_uri(state: &State, uri: Uri) -> Result<Uri, String> {
    if uri.is_absolute() {
        Ok(uri)
    } else {
        match Headers::borrow_from(&state).get::<header::Host>() {
            Some(host) => host_to_uri(host, &uri.to_string()[..]),
            None => return Err("No host header found".into()),
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
    use hyper::{Headers, Uri};
    use hyper::header::Host;
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
            let mut headers = Headers::new();
            headers.set(Host::new(String::from("example.com"), None));
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
            let mut headers = Headers::new();
            headers.set(Host::new(String::from("example.com"), Some(1234)));
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
