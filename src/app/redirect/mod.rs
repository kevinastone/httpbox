mod uri;

use self::uri::absolute_url;
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

pub fn to(state: State) -> (State, Response<Body>) {
    let query = RedirectUrlParams::borrow_from(&state);
    let url = try_or_error_response!(state, Url::parse(&query.url));
    redirect_to(state, &url.to_string())
}

pub fn relative(state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::borrow_from(&state).n;
    n = min(n - 1, 100);

    let url = if n > 0 {
        format!("/relative-redirect/{}", n)
    } else {
        String::from("/")
    };

    redirect_to(state, &url)
}

pub fn redirect(state: State) -> (State, Response<Body>) {
    relative(state)
}

pub fn absolute(mut state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::borrow_from(&state).n;
    n = min(n - 1, 100);

    let url = if n > 0 {
        format!("/absolute-redirect/{}", n)
    } else {
        String::from("/")
    };

    let request_uri = Uri::take_from(&mut state);
    let response_url = try_or_error_response!(
        state,
        absolute_url(&state, request_uri).and_then(|base| Ok(base.join(&url)?))
    );
    redirect_to(state, &response_url.to_string())
}

#[cfg(test)]
mod test {
    use crate::app::app;

    use gotham::test::TestServer;
    use http::header;
    use http::StatusCode;

    #[test]
    fn test_redirect_to() {
        let test_server = TestServer::new(app()).unwrap();
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
    fn test_invalid_redirect_to() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect-to?url=abc")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_redirect() {
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
