extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::status;
use self::router::Router;
use std::cmp::min;
use std::thread;
use std::time::Duration;

fn sleep(seconds: u64) {
    // Only delay when not testing
    if !(cfg!(test)) {
        thread::sleep(Duration::from_secs(seconds));
    }
}

pub fn delay(req: &mut Request) -> IronResult<Response> {

    let delay = req.extensions
        .get::<Router>()
        .unwrap()
        .find("n")
        .unwrap_or("10");
    let delay = itry!(delay.parse::<u64>(), status::BadRequest);
    let delay = min(delay, 10);

    sleep(delay);

    Ok(Response::with((status::Ok, delay.to_string())))
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use iron::status;
    use self::iron_test::{request, response};

    #[test]
    fn test_sleep() {

        let app = app();

        let res = request::get("http://localhost:3000/delay/3", Headers::new(), &app).unwrap();

        assert_eq!(res.status.unwrap(), status::Ok);

        let result_body = response::extract_body_to_string(res);
        assert_eq!(result_body, "3")
    }
}
