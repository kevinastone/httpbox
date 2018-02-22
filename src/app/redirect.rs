extern crate iron;
extern crate router;
extern crate urlencoded;

use self::iron::{IronResult, Request, Response, Url as IronUrl};
use self::iron::Plugin;
use self::iron::headers;
use self::iron::modifiers::{Header, Redirect};
use self::iron::status;
use self::iron::url::Url as RustUrl;
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;

use std::env;

const URL_QUERY_PARAM: &'static str = "url";

lazy_static! {
    static ref BASE_URL: Option<RustUrl> =
        env::var_os("BASE_URL")
            .and_then(|os_str| os_str.into_string().ok())
            .and_then(|url| RustUrl::parse(&url).ok());
}

pub fn absolute_url<'a, 'b>(url: &'b str, base: &'a RustUrl) -> Option<IronUrl> {
    base.join(url)
        .map_err(|e| e.to_string())
        .and_then(|url| IronUrl::from_generic_url(url))
        .ok()
}

pub fn to(req: &mut Request) -> IronResult<Response> {
    let url = iexpect!(
        req.get_ref::<UrlEncodedQuery>()
            .ok()
            .and_then(|hashmap| hashmap.get(URL_QUERY_PARAM))
            .and_then(|vals| vals.first())
            .and_then(|url| IronUrl::parse(url).ok()),
        status::BadRequest
    );

    Ok(Response::with((status::Found, Redirect(url))))
}

pub fn relative(req: &mut Request) -> IronResult<Response> {
    let mut code = itry!(
        req.extensions
            .get::<Router>()
            .unwrap()
            .find("n")
            .unwrap_or("1")
            .parse::<u16>(),
        status::BadRequest
    );

    code = code - 1;

    let url = if code <= 0 {
        String::from("/")
    } else {
        format!("/relative-redirect/{}", code)
    };

    Ok(Response::with((
        status::Found,
        Header(headers::Location(url)),
    )))
}

pub fn redirect(req: &mut Request) -> IronResult<Response> {
    relative(req)
}

pub fn absolute(req: &mut Request) -> IronResult<Response> {
    let mut code = itry!(
        req.extensions
            .get::<Router>()
            .unwrap()
            .find("n")
            .unwrap_or("1")
            .parse::<u16>(),
        status::BadRequest
    );

    code = code - 1;

    let url = if code <= 0 {
        String::from("/")
    } else {
        format!("/absolute-redirect/{}", code)
    };

    let base: RustUrl = BASE_URL.clone().unwrap_or_else(|| req.url.clone().into());
    let url = iexpect!(absolute_url(&url[..], &base), status::BadRequest);
    Ok(Response::with((status::Found, Redirect(url))))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::absolute_url;
    use super::iron::headers;
    use super::iron::url::Url as RustUrl;
    use super::super::app;
    use iron::Headers;
    use iron::status;
    use self::iron_test::request;

    #[test]
    fn test_absolute_url() {
        let base = RustUrl::parse("https://example.com").unwrap();
        assert_eq!(
            absolute_url("/something", &base).unwrap().to_string(),
            "https://example.com/something"
        );
    }

    #[test]
    fn test_redirect_to() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/redirect-to?url=http://example.com",
            Headers::new(),
            &app,
        ).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "http://example.com/")
    }

    #[test]
    fn test_redirect() {
        let app = app();

        let res = request::get("http://localhost:3000/redirect/5", Headers::new(), &app).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/relative-redirect/4")
    }

    #[test]
    fn test_redirect_last() {
        let app = app();

        let res = request::get("http://localhost:3000/redirect/1", Headers::new(), &app).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/")
    }

    #[test]
    fn test_relative_redirect() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/relative-redirect/5",
            Headers::new(),
            &app,
        ).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/relative-redirect/4")
    }

    #[test]
    fn test_relative_redirect_last() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/relative-redirect/1",
            Headers::new(),
            &app,
        ).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/")
    }

    #[test]
    fn test_absolute_redirect() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/absolute-redirect/5",
            Headers::new(),
            &app,
        ).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "http://localhost:3000/absolute-redirect/4")
    }

    #[test]
    fn test_absolute_redirect_last() {
        let app = app();

        let res = request::get(
            "http://localhost:3000/absolute-redirect/1",
            Headers::new(),
            &app,
        ).unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "http://localhost:3000/")
    }
}
