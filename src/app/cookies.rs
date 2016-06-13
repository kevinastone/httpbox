extern crate iron;
extern crate lazy_static;
extern crate urlencoded;

use self::iron::Plugin;
use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::UrlEncodedQuery;

lazy_static! {
    static ref EMPTY_COOKIES: headers::Cookie = headers::Cookie(Vec::new());
    static ref EMPTY_QUERYMAP: QueryMap = QueryMap::new();
}


pub fn cookies(req: &mut Request) -> IronResult<Response> {
    let cookies = req.headers
        .get::<headers::Cookie>()
        .unwrap_or(&EMPTY_COOKIES)
        .iter()
        .map(|c| format!("{}", c))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(Response::with((status::Ok, cookies.to_string())))
}

pub fn set_cookies(req: &mut Request) -> IronResult<Response> {
    let cookies = req.get_ref::<UrlEncodedQuery>()
        .ok()
        .unwrap_or(&EMPTY_QUERYMAP)
        .iter()
        .map(|(k, v)| headers::CookiePair::new(k.to_owned(), v.first().unwrap().to_owned()))
        .collect::<Vec<headers::CookiePair>>();

    let cookies = headers::SetCookie(cookies);

    Ok(Response::with((status::Ok, Header(cookies))))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use super::iron::headers;
    use iron::Headers;
    use self::iron_test::{request, response};

    #[test]
    fn test_no_cookies() {

        let app = app();

        let res = request::get("http://localhost:3000/cookies", Headers::new(), &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "")
    }

    #[test]
    fn test_cookies() {

        let app = app();

        let mut headers = Headers::new();
        headers.set(headers::Cookie(vec![headers::CookiePair::new("test".to_owned(),
                                                                  "value".to_owned())]));

        let res = request::get("http://localhost:3000/cookies", headers, &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "test=value")
    }

    #[test]
    fn test_set_cookies() {

        let app = app();

        let res = request::get("http://localhost:3000/cookies/set?test=value",
                               Headers::new(),
                               &app)
            .unwrap();

        let cookies = res.headers.get::<headers::SetCookie>().unwrap();
        let cookie = cookies.0.first().unwrap();

        assert_eq!(cookie.name, "test");
        assert_eq!(cookie.value, "value");
    }
}
