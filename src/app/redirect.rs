extern crate iron;
extern crate router;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult, Url};
use self::iron::Plugin;
use self::iron::headers;
use self::iron::modifiers::{Redirect, Header};
use self::iron::status;
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;


const URL_QUERY_PARAM: &'static str = "url";


pub fn to(req: &mut Request) -> IronResult<Response> {
    let url = iexpect!(req.get_ref::<UrlEncodedQuery>()
                           .ok()
                           .and_then(|hashmap| hashmap.get(URL_QUERY_PARAM))
                           .and_then(|vals| vals.first())
                           .and_then(|url| Url::parse(url).ok()),
                       status::BadRequest);

    Ok(Response::with((status::Found, Redirect(url))))
}

pub fn relative(req: &mut Request) -> IronResult<Response> {

    let mut code = itry!(req.extensions
                             .get::<Router>()
                             .unwrap()
                             .find("n")
                             .unwrap_or("1")
                             .parse::<u16>(),
                         status::BadRequest);

    code = code - 1;

    let url = if code <= 0 {
        String::from("/")
    } else {
        format!("/relative-redirect/{}", code)
    };

    Ok(Response::with((status::Found, Header(headers::Location(url)))))
}

pub fn redirect(req: &mut Request) -> IronResult<Response> {

    relative(req)
}

pub fn absolute(req: &mut Request) -> IronResult<Response> {

    let mut code = itry!(req.extensions
                             .get::<Router>()
                             .unwrap()
                             .find("n")
                             .unwrap_or("1")
                             .parse::<u16>(),
                         status::BadRequest);

    code = code - 1;

    let url = if code <= 0 {
        String::from("/")
    } else {
        format!("/absolute-redirect/{}", code)
    };

    let url = iexpect!(req.url
                           .clone()
                           .into_generic_url()
                           .join(&url[..])
                           .map_err(|e| e.to_string())
                           .and_then(|url| Url::from_generic_url(url))
                           .ok(),
                       status::BadRequest);

    Ok(Response::with((status::Found, Redirect(url))))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use super::iron::headers;
    use iron::Headers;
    use iron::status;
    use self::iron_test::request;

    #[test]
    fn test_redirect_to() {

        let app = app();

        let res = request::get("http://localhost:3000/redirect-to?url=http://example.com",
                               Headers::new(),
                               &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        // TODO: Fix with new version of Iron that doesn't include the port
        assert_eq!(location.0, "http://example.com:80/")
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

        let res = request::get("http://localhost:3000/relative-redirect/5",
                               Headers::new(),
                               &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/relative-redirect/4")
    }

    #[test]
    fn test_relative_redirect_last() {

        let app = app();

        let res = request::get("http://localhost:3000/relative-redirect/1",
                               Headers::new(),
                               &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "/")
    }

    #[test]
    fn test_absolute_redirect() {

        let app = app();

        let res = request::get("http://localhost:3000/absolute-redirect/5",
                               Headers::new(),
                               &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "http://localhost:3000/absolute-redirect/4")
    }

    #[test]
    fn test_absolute_redirect_last() {

        let app = app();

        let res = request::get("http://localhost:3000/absolute-redirect/1",
                               Headers::new(),
                               &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Found);

        let location = res.headers.get::<headers::Location>().unwrap();
        assert_eq!(location.0, "http://localhost:3000/")
    }
}
