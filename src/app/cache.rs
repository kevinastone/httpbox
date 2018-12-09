use crate::app::response::{empty_response, ok};
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};

use http::header;
use hyper::{Body, HeaderMap, Response, StatusCode};
use hyperx::header as headerx;
use serde_derive::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct CacheTimeParams {
    n: u32,
}

pub fn cache(mut state: State) -> (State, Response<Body>) {
    let headers = HeaderMap::take_from(&mut state);
    if headers.get(header::IF_MODIFIED_SINCE).is_some()
        || headers.get(header::IF_NONE_MATCH).is_some()
    {
        let res = empty_response(&state, StatusCode::NOT_MODIFIED);
        (state, res)
    } else {
        ok(state, vec![])
    }
}

pub fn set_cache(mut state: State) -> (State, Response<Body>) {
    let n = CacheTimeParams::take_from(&mut state).n;

    let mut res = empty_response(&state, StatusCode::OK);
    {
        let headers = res.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_str(
                &headerx::CacheDirective::MaxAge(n).to_string(),
            )
            .unwrap(),
        );
    }
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use http::header;
    use hyper::StatusCode;
    use hyperx::header::{CacheDirective, HttpDate};
    use std::time::SystemTime;

    #[test]
    fn test_cache_no_headers() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_cache_if_modified_since() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(
                header::IF_MODIFIED_SINCE,
                header::HeaderValue::from_str(
                    &HttpDate::from(SystemTime::now()).to_string(),
                )
                .unwrap(),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[test]
    fn test_cache_if_none_match() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(
                header::IF_NONE_MATCH,
                header::HeaderValue::from_static("*"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[test]
    fn test_set_cache() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache/30")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            header::HeaderValue::from_str(
                &CacheDirective::MaxAge(30).to_string()
            )
            .unwrap()
        )
    }
}
