extern crate iron;
extern crate lazy_static;
extern crate urlencoded;

use self::iron::{IronResult, Request, Response};
use self::iron::Plugin;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::UrlEncodedQuery;

lazy_static! {
    static ref EMPTY_QUERYMAP: QueryMap = QueryMap::new();
}

pub fn headers(req: &mut Request) -> IronResult<Response> {
    let headers = req.headers
        .iter()
        .map(|h| format!("{}", h).trim().to_owned())
        .collect::<Vec<String>>()
        .join("\n");
    Ok(Response::with((status::Ok, headers.to_string())))
}

pub fn response_headers(req: &mut Request) -> IronResult<Response> {
    let headers = req.get_ref::<UrlEncodedQuery>()
        .ok()
        .unwrap_or(&EMPTY_QUERYMAP);

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
        assert!(result_body.contains("X-Request-ID: 1234"));
        assert_eq!(
            result_body,
            "Content-Length: 0\nX-Request-ID: 1234\nUser-Agent: iron-test"
        )
    }

    #[test]
    fn test_response_headers() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/response-headers?X-Request-ID=1234",
            Headers::new(),
            &app,
        ).unwrap();
        assert_eq!(
            res.headers
                .get_raw("X-Request-ID")
                .map(|v| &v[0])
                .and_then(|v| String::from_utf8(v.clone()).ok())
                .unwrap(),
            "1234"
        )
    }
}
