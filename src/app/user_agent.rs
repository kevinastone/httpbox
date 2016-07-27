extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::headers::UserAgent;
use self::iron::status;

pub fn user_agent(req: &mut Request) -> IronResult<Response> {

    let user_agent = iexpect!(req.headers.get::<UserAgent>());
    Ok(Response::with((status::Ok, user_agent.to_string())))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use super::iron::headers;
    use iron::Headers;
    use self::iron_test::{request, response};

    #[test]
    fn test_user_agent() {

        let app = app();

        let res = request::get("http://localhost:3000/user-agent", Headers::new(), &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "iron-test")
    }

    #[test]
    fn test_user_agent_custom() {

        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::UserAgent("CustomAgent/1.0".to_owned()));

        let res = request::get("http://localhost:3000/user-agent", headers, &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "CustomAgent/1.0")
    }
}
