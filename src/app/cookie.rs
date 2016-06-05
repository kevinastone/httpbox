extern crate iron;

use self::iron::headers;

pub fn parse_cookies<'a>(headers: &headers::Headers) -> headers::Cookie {

    headers.get::<headers::Cookie>()
        .and_then(|c| Some(c.clone()))
        .unwrap_or_else(|| headers::Cookie(Vec::new()))
}
