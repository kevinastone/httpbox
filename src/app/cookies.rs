extern crate iron;
extern crate urlencoded;

use self::iron::Plugin;
use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::UrlEncodedQuery;

pub fn cookies(req: &mut Request) -> IronResult<Response> {
    let cookies = req.headers
        .get::<headers::Cookie>()
        .and_then(|c| Some(c.clone()))
        .unwrap_or_else(|| headers::Cookie(Vec::new()))
        .iter()
        .map(|c| format!("{}", c))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(Response::with((status::Ok, cookies.to_string())))
}

pub fn set_cookies(req: &mut Request) -> IronResult<Response> {
    let cookies = req.get_ref::<UrlEncodedQuery>()
        .ok()
        .and_then(|c| Some(c.clone()))
        .unwrap_or_else(|| QueryMap::new())
        .iter()
        .map(|(k, v)| headers::CookiePair::new(k.to_owned(), v.first().unwrap().to_owned()))
        .collect::<Vec<headers::CookiePair>>();

    let cookies = headers::SetCookie(cookies);

    Ok(Response::with((status::Ok, Header(cookies))))
}
