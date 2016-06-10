extern crate iron;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult};
use self::iron::Plugin;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::UrlEncodedQuery;

pub fn headers(req: &mut Request) -> IronResult<Response> {
    let headers = req.headers.iter().map(|h| format!("{}", h)).collect::<Vec<String>>().join("\n");
    Ok(Response::with((status::Ok, headers.to_string())))
}

pub fn response_headers(req: &mut Request) -> IronResult<Response> {
    let headers = req.get_ref::<UrlEncodedQuery>()
        .ok()
        .and_then(|c| Some(c.clone()))
        .unwrap_or_else(|| QueryMap::new());

    let mut res = Response::with(status::Ok);
    for (name, value) in headers {

        let encoded_vals = value.iter().map(|s| s.clone().into_bytes()).collect();
        res.headers.set_raw(name.to_owned(), encoded_vals);
    }

    Ok(res)
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use self::iron_test::{request, response};

    #[test]
    fn test_headers() {

        let app = app();

        let mut headers = Headers::new();
        headers.set_raw("X-Request-ID", vec![String::from("1234").into_bytes()]);

        let res = request::get("http://localhost:3000/headers", headers, &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert!(result_body.contains("X-Request-ID: 1234"))
    }

    #[test]
    fn test_response_headers() {

        let app = app();

        let res = request::get("http://localhost:3000/response-headers?X-Request-ID=1234",
                               Headers::new(),
                               &app)
            .unwrap();
        assert_eq!(res.headers
                       .get_raw("X-Request-ID")
                       .map(|v| &v[0])
                       .and_then(|v| String::from_utf8(v.clone()).ok())
                       .unwrap(),
                   "1234")
    }
}
