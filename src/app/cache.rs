extern crate gotham;
extern crate hyper;
extern crate mime;

use app::response::ok;
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{header, Headers, Response, StatusCode};

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct CacheTimeParams {
    n: u32,
}

pub fn cache(mut state: State) -> (State, Response) {
    let headers = Headers::take_from(&mut state);
    if headers.get::<header::IfModifiedSince>().is_some()
        || headers.get::<header::IfNoneMatch>().is_some()
    {
        let res = create_response(
            &state,
            StatusCode::NotModified,
            Some((vec![], mime::TEXT_PLAIN)),
        );
        (state, res)
    } else {
        ok(state, vec![])
    }
}

pub fn set_cache(mut state: State) -> (State, Response) {
    let n = CacheTimeParams::take_from(&mut state).n;

    let mut res = create_response(
        &state,
        StatusCode::Ok,
        Some((vec![], mime::TEXT_PLAIN)),
    );
    {
        let headers = res.headers_mut();
        headers.set(header::CacheControl(vec![
            header::CacheDirective::MaxAge(n),
        ]))
    }
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;
    use hyper::header::{CacheControl, CacheDirective, HttpDate,
                        IfModifiedSince, IfNoneMatch};
    use std::time::SystemTime;

    #[test]
    fn test_cache_no_headers() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
    }

    #[test]
    fn test_cache_if_modified_since() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(IfModifiedSince(HttpDate::from(SystemTime::now())))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NotModified);
    }

    #[test]
    fn test_cache_if_none_match() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache")
            .with_header(IfNoneMatch::Any)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::NotModified);
    }

    #[test]
    fn test_set_cache() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cache/30")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(
            response.headers().get::<CacheControl>().unwrap(),
            &CacheControl(vec![CacheDirective::MaxAge(30)])
        )
    }
}
