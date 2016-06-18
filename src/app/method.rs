extern crate iron;
extern crate lazy_static;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult};
use self::iron::Plugin;
use self::iron::status;
use self::urlencoded::QueryMap;
use self::urlencoded::UrlEncodedQuery;

lazy_static! {
    static ref EMPTY_QUERYMAP: QueryMap = QueryMap::new();
}

pub fn get(req: &mut Request) -> IronResult<Response> {

    let mut query_params: Vec<String> = vec![];
    for (key, values) in req.get_ref::<UrlEncodedQuery>()
        .ok()
        .unwrap_or(&EMPTY_QUERYMAP)
        .iter() {
        for value in values {
            query_params.push(format!("{}={}", key, value))
        }
    }

    Ok(Response::with((status::Ok, query_params.join("\n"))))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use self::iron_test::{request, response};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_get() {

        let app = app();

        let res = request::get("http://localhost:3000/get?key=val", Headers::new(), &app).unwrap();

        let result_body = response::extract_body_to_string(res);
        assert!(result_body.contains("key=val"))
    }

    #[test]
    fn test_multi_get() {

        let app = app();

        let res = request::get("http://localhost:3000/get?key=val&other=something&key=another",
                               Headers::new(),
                               &app)
            .unwrap();

        let result_body = response::extract_body_to_string(res);
        let result: HashSet<&str> = HashSet::from_iter(result_body.split("\n"));
        let expected = HashSet::from_iter(vec!["key=val", "key=another", "other=something"]);
        assert_eq!(expected, result)
    }
}
