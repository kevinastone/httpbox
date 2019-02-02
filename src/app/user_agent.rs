use crate::app::response::ok;
use crate::headers::{HeaderMapExt, UserAgent};
use gotham::state::{FromState, State};
use hyper::{Body, HeaderMap, Response};

pub fn user_agent(state: State) -> (State, Response<Body>) {
    let user_agent = eexpect!(
        state,
        HeaderMap::borrow_from(&state)
            .typed_get::<UserAgent>()
            .map(|ua| ua.to_string())
    );
    ok(state, user_agent)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use crate::headers::UserAgent;
    use crate::test::request::TestRequestTypedHeader;
    use gotham::test::TestServer;
    use http::StatusCode;

    #[test]
    fn test_user_agent() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_user_agent_custom() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .with_typed_header(UserAgent::from_static("HTTPBoxBot/1.0"))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "HTTPBoxBot/1.0");
    }
}
