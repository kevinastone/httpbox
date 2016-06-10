extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

pub fn ip(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, req.remote_addr.ip().to_string())))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use self::iron_test::{request, response};

    #[test]
    fn test_ip() {

        let app = app();

        let res = request::get("http://localhost:3000/ip", Headers::new(), &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "127.0.0.1")
    }
}
