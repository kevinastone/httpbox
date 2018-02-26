extern crate gotham;
extern crate hyper;
extern crate mime;

mod body;

use app::response::ok;
use gotham::state::{FromState, State};
use gotham::handler::HandlerFuture;
use hyper::{Response, Uri};
use self::body::parse_body;
use url::form_urlencoded;

pub fn get(state: State) -> (State, Response) {
    let params: Vec<String> = {
        Uri::borrow_from(&state)
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .map(|pairs| {
                pairs
                    .map(|(key, value)| format!("{} = {}", key, value))
                    .collect()
            })
            .unwrap_or_else(|| vec![])
    };

    ok(state, params.join("\n").into_bytes())
}

pub fn post(state: State) -> Box<HandlerFuture> {
    parse_body(state)
}

pub fn put(state: State) -> Box<HandlerFuture> {
    parse_body(state)
}

pub fn patch(state: State) -> Box<HandlerFuture> {
    parse_body(state)
}

pub fn delete(state: State) -> Box<HandlerFuture> {
    parse_body(state)
}

#[cfg(test)]
mod test {
    use super::mime;
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_get() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/get?key=val")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val");
    }

    #[test]
    fn test_multi_get() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get(
                "http://localhost:3000/get?key=val&other=something&key=another",
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .post(
                "http://localhost:3000/post",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_post() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .post(
                "http://localhost:3000/post",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .put(
                "http://localhost:3000/put",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();

        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_put() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .put(
                "http://localhost:3000/put",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .patch(
                "http://localhost:3000/patch",
                "key=val",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_patch() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .patch(
                "http://localhost:3000/patch",
                "key=val&other=something&key=another",
                mime::APPLICATION_WWW_FORM_URLENCODED,
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .delete("http://localhost:3000/delete")
            .perform()
            .unwrap();
        assert_eq!(response.status(), StatusCode::Ok);
    }
}
