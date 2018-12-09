mod uri;

use self::uri::{absolute_url, join_url};
use crate::app::response::redirect_to;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use hyper::{Body, Response, Uri};
use serde_derive::Deserialize;
use std::cmp::min;
use url::Url;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectCountParams {
    n: u16,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectUrlParams {
    url: String,
}

pub fn to(mut state: State) -> (State, Response<Body>) {
    let query = RedirectUrlParams::take_from(&mut state);
    let url = try_or_error_response!(state, Url::parse(&query.url[..]));
    redirect_to(state, &url.to_string())
}

pub fn relative(mut state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::take_from(&mut state).n;
    n = min(n - 1, 100);

    let url = if n == 0 {
        String::from("/")
    } else {
        format!("/relative-redirect/{}", n)
    };

    redirect_to(state, &url)
}

pub fn redirect(state: State) -> (State, Response<Body>) {
    relative(state)
}

pub fn absolute(mut state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::take_from(&mut state).n;
    n = min(n - 1, 100);

    let url = if n == 0 {
        String::from("/")
    } else {
        format!("/absolute-redirect/{}", n)
    };

    let uri = Uri::borrow_from(&state).clone();
    let base = absolute_url(&state, uri);
    let url = expect_or_error_response!(
        state,
        base.and_then(|base| join_url(&url[..], &base))
    );
    redirect_to(state, &url.to_string())
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use http::header;
    use hyper::StatusCode;

    #[test]
    fn test_redirect_to() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect-to?url=http://example.com")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("http://example.com/")
        )
    }

    #[test]
    fn test_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("/relative-redirect/4")
        )
    }

    #[test]
    fn test_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("/")
        )
    }

    #[test]
    fn test_relative_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("/relative-redirect/4")
        )
    }

    #[test]
    fn test_relative_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("/")
        )
    }

    #[test]
    fn test_absolute_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static(
                "http://localhost:3000/absolute-redirect/4"
            )
        )
    }

    #[test]
    fn test_absolute_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            header::HeaderValue::from_static("http://localhost:3000/")
        )
    }
}
