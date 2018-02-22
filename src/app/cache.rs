extern crate iron;
extern crate router;

use self::iron::{IronResult, Request, Response};
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::router::Router;

pub fn cache(req: &mut Request) -> IronResult<Response> {
    if req.headers.get::<headers::IfModifiedSince>().is_some()
        || req.headers.get::<headers::IfNoneMatch>().is_some()
    {
        Ok(Response::with(status::Status::NotModified))
    } else {
        Ok(Response::with(status::Status::Ok))
    }
}

pub fn set_cache(req: &mut Request) -> IronResult<Response> {
    let n = iexpect!(req.extensions.get::<Router>().unwrap().find("n"));
    let n = itry!(n.parse::<u32>(), status::BadRequest);

    Ok(Response::with((
        status::Status::Ok,
        Header(headers::CacheControl(vec![
            headers::CacheDirective::MaxAge(n),
        ])),
    )))
}

#[cfg(test)]
mod test {

    extern crate iron_test;
    extern crate time;

    use super::super::app;
    use super::iron::headers;
    use iron::Headers;
    use iron::status;
    use self::iron_test::request;

    #[test]
    fn test_cache_no_headers() {
        let app = app();

        let res = request::get("http://localhost:3000/cache", Headers::new(), &app).unwrap();

        assert_eq!(res.status.unwrap(), status::Ok)
    }

    #[test]
    fn test_cache_if_modified_since() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::IfModifiedSince(headers::HttpDate(time::now())));

        let res = request::get("http://localhost:3000/cache", headers, &app).unwrap();

        assert_eq!(res.status.unwrap(), status::NotModified)
    }

    #[test]
    fn test_cache_if_none_match() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::IfNoneMatch::Any);

        let res = request::get("http://localhost:3000/cache", headers, &app).unwrap();

        assert_eq!(res.status.unwrap(), status::NotModified)
    }

    #[test]
    fn test_set_cache() {
        let app = app();

        let res = request::get("http://localhost:3000/cache/30", Headers::new(), &app).unwrap();

        let cache_control = res.headers.get::<headers::CacheControl>().unwrap();
        match cache_control.0.first().unwrap() {
            &headers::CacheDirective::MaxAge(n) => assert_eq!(n, 30),
            _ => panic!("Invalid cache-control header"),
        }
    }
}
