extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::status;
use self::router::Router;

pub fn status_code(req: &mut Request) -> IronResult<Response> {

    let code = req.extensions.get::<Router>().unwrap().find("code").unwrap_or("200");
    let code = itry!(code.parse::<u16>(), status::BadRequest);

    Ok(Response::with(status::Status::from_u16(code)))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use iron::status;
    use self::iron_test::request;

    #[test]
    fn test_status_code() {

        let app = app();

        let res = request::get("http://localhost:3000/status/429", Headers::new(), &app).unwrap();

        assert_eq!(res.status.unwrap(), status::TooManyRequests);
    }
}
