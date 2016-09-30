extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::status;

pub const X_FORWARD_FOR: &'static str = "X-Forwarded-For";


pub fn ip(req: &mut Request) -> IronResult<Response> {
    let remote_ip = iexpect!(req.headers
        .get_raw(X_FORWARD_FOR)
        .and_then(|h| headers::parsing::from_one_raw_str(h).ok())
        .or_else(|| Some(req.remote_addr.ip().to_string())));

    Ok(Response::with((status::Ok, remote_ip)))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::X_FORWARD_FOR;
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

    #[test]
    fn test_ip_from_header() {

        let app = app();

        let mut headers = Headers::new();
        headers.set_raw(X_FORWARD_FOR, vec![String::from("1.2.3.4").into_bytes()]);
        let res = request::get("http://localhost:3000/ip", headers, &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "1.2.3.4")
    }
}
