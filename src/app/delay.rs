use crate::app::response::ok;
use futures::{future, Future};
use futures_timer::Delay;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use serde_derive::Deserialize;
use std::cmp::min;
use std::time::Duration;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct DelayParams {
    n: u64,
}

fn sleep_duration(seconds: u64) -> u64 {
    // Only delay when not testing
    if cfg!(test) {
        return 0;
    }
    seconds
}

pub fn delay(state: State) -> Box<HandlerFuture> {
    let params = DelayParams::borrow_from(&state);
    let delay = min(params.n, 10);

    let f = Delay::new(Duration::from_secs(sleep_duration(delay))).then(
        move |result| {
            future_try_or_error_response!(state, result);
            future::ok(ok(state, format!("{}", delay)))
        },
    );

    Box::new(f)
}

#[cfg(test)]
mod test {
    use super::super::router;
    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_sleep() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/delay/3")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "3");
    }

    #[test]
    fn test_sleep_too_long() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/delay/33")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "10");
    }
}
