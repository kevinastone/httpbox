extern crate gotham;
extern crate hyper;
extern crate mime;

use app::response::{bad_request, ok};
use gotham::state::{FromState, State};

use hyper::{Headers, Response};
use hyper::header::UserAgent;

pub fn user_agent(state: State) -> (State, Response) {
    match Headers::borrow_from(&state)
        .get::<UserAgent>()
        .map(|ua| ua.to_string())
    {
        Some(user_agent) => ok(state, user_agent.into_bytes()),
        None => bad_request(state),
    }
}

#[cfg(test)]
mod test {
    use super::UserAgent;
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_user_agent() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BadRequest);
    }

    #[test]
    fn test_user_agent_custom() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .with_header(UserAgent::new(String::from("HTTPBoxBot/1.0")))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "HTTPBoxBot/1.0");
    }
}
