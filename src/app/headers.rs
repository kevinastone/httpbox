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
