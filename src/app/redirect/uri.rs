use failure::{bail, Fallible};
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

pub fn join_url(url: &str, base: &Url) -> Option<Url> {
    base.join(url).ok()
}

fn host_to_uri(host: &str, relative_uri: &str) -> Fallible<Uri> {
    Ok(format!(
        "{}://{}",
        "http", // FIXME: Determine protocol
        host,
    )
    .parse::<Url>()?
    .join(relative_uri)?
    .to_string()
    .parse()?)
}

fn absolute_uri(state: &State, uri: Uri) -> Fallible<Uri> {
    if uri.scheme_part().is_some() {
        Ok(uri)
    } else {
        match HeaderMap::borrow_from(&state)
            .get(header::HOST)
            .and_then(|host| host.to_str().ok())
        {
            Some(host) => host_to_uri(host, &uri.to_string()),
            None => bail!("No host header found"),
        }
    }
}

pub fn absolute_url(state: &State, uri: Uri) -> Option<Url> {
    BASE_URL
        .clone()
        .or_else(|| absolute_uri(&state, uri).ok()?.to_string().parse().ok())
}

#[cfg(test)]
mod test {
    use super::{absolute_uri, join_url};

    use gotham::state::State;
    use headers_ext::{HeaderMapExt, Host};
    use http::uri::Authority;
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
