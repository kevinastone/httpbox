extern crate bodyparser;
extern crate iron;
extern crate lazy_static;
extern crate urlencoded;

use self::iron::{IronResult, Request, Response};
use self::iron::headers;
use self::iron::mime;
use self::iron::Plugin;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::{UrlEncodedBody, UrlEncodedQuery};

lazy_static! {
    static ref EMPTY_QUERYMAP: QueryMap = QueryMap::new();
}

fn parse_raw_body(req: &mut Request) -> IronResult<Response> {
    let body = itry!(req.get::<bodyparser::Raw>()).unwrap_or_else(|| String::from(""));
    Ok(Response::with((status::Ok, body)))
}

fn parse_url_encoded_body(req: &mut Request) -> IronResult<Response> {
    let mut body_params: Vec<String> = vec![];
    for (key, values) in req.get_ref::<UrlEncodedBody>()
        .ok()
        .unwrap_or(&EMPTY_QUERYMAP)
        .iter()
    {
        body_params.push(format!("{} = {}", key, values.join(", ")))
    }

    Ok(Response::with((status::Ok, body_params.join("\n"))))
}

fn parse_body(req: &mut Request) -> IronResult<Response> {
    let headers = req.headers.clone();
    let content_type = headers.get::<headers::ContentType>();

    match content_type {
        Some(content_type) => match content_type {
            &headers::ContentType(mime::Mime(
                mime::TopLevel::Application,
                mime::SubLevel::WwwFormUrlEncoded,
                ..
            )) => parse_url_encoded_body(req),
            _ => parse_raw_body(req),
        },
        _ => parse_raw_body(req),
    }
}

pub fn get(req: &mut Request) -> IronResult<Response> {
    let mut query_params: Vec<String> = vec![];
    for (key, values) in req.get_ref::<UrlEncodedQuery>()
        .ok()
        .unwrap_or(&EMPTY_QUERYMAP)
        .iter()
    {
        query_params.push(format!("{} = {}", key, values.join(", ")))
    }

    Ok(Response::with((status::Ok, query_params.join("\n"))))
}

pub fn post(req: &mut Request) -> IronResult<Response> {
    parse_body(req)
}

pub fn put(req: &mut Request) -> IronResult<Response> {
    parse_body(req)
}

pub fn patch(req: &mut Request) -> IronResult<Response> {
    parse_body(req)
}

pub fn delete(req: &mut Request) -> IronResult<Response> {
    parse_body(req)
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use iron::headers;
    use iron::status;
    use self::iron_test::{request, response};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_get() {
        let app = app();

        let res = request::get("http://localhost:3000/get?key=val", Headers::new(), &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_get() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/get?key=val&other=something&key=another",
            Headers::new(),
            &app,
        ).unwrap();

        let result_body = response::extract_body_to_string(res);
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec!["key = val, another", "other = something"]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_post() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::post("http://localhost:3000/post", headers, "key=val", &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_post() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::post(
            "http://localhost:3000/post",
            headers,
            "key=val&other=something&key=another",
            &app,
        ).unwrap();

        let result_body = response::extract_body_to_string(res);
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec!["key = val, another", "other = something"]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_put() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::put("http://localhost:3000/put", headers, "key=val", &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_put() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::put(
            "http://localhost:3000/put",
            headers,
            "key=val&other=something&key=another",
            &app,
        ).unwrap();

        let result_body = response::extract_body_to_string(res);
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec!["key = val, another", "other = something"]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_patch() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::patch("http://localhost:3000/patch", headers, "key=val", &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "key = val")
    }

    #[test]
    fn test_multi_patch() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::patch(
            "http://localhost:3000/patch",
            headers,
            "key=val&other=something&key=another",
            &app,
        ).unwrap();

        let result_body = response::extract_body_to_string(res);
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec!["key = val, another", "other = something"]);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_delete() {
        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::ContentType::form_url_encoded());
        let res = request::delete("http://localhost:3000/delete", headers, &app).unwrap();
        assert_eq!(res.status.unwrap(), status::Ok)
    }
}
