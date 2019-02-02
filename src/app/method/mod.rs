mod body;

use self::body::body;
use crate::app::response::ok;
use crate::http::{Response, Uri};
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use itertools::{Either, Itertools};
use url::form_urlencoded;

pub fn get(state: State) -> (State, Response) {
    let body = {
        Uri::borrow_from(&state)
            .query()
            .map_or_else(
                || Either::Right(vec![]),
                |query| Either::Left(form_urlencoded::parse(query.as_bytes())),
            )
            .into_iter()
            .format_with("\n", |(key, value), f| {
                f(&format_args!("{} = {}", key, value))
            })
            .to_string()
    };

    ok(state, body)
}

pub fn post(state: State) -> Box<HandlerFuture> {
    body(state)
}

pub fn put(state: State) -> Box<HandlerFuture> {
    body(state)
}

pub fn patch(state: State) -> Box<HandlerFuture> {
    body(state)
}

pub fn delete(state: State) -> Box<HandlerFuture> {
    body(state)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use gotham::test::TestServer;
    use http::StatusCode;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_get() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/get?key=val")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val");
    }

    #[test]
    fn test_multi_get() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get(
                "http://localhost:3000/get?key=val&other=something&key=another",
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_post() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .post(
                "http://localhost:3000/post",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_post() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .post(
                "http://localhost:3000/post",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_put() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .put(
                "http://localhost:3000/put",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();

        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_put() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .put(
                "http://localhost:3000/put",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();

        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_patch() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .patch(
                "http://localhost:3000/patch",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_patch() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .patch(
                "http://localhost:3000/patch",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();

        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_delete() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .delete("http://localhost:3000/delete")
            .perform()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
