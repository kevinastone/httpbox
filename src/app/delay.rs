use crate::app::response::ok;
use futures::prelude::*;
use futures_timer::Delay;
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use serde_derive::Deserialize;
use std::cmp::min;
use std::time::Duration;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct DelayParams {
    n: u64,
}

#[cfg(test)]
fn sleep_duration(_: u64) -> u64 {
    0
}

#[cfg(not(test))]
#[inline]
fn sleep_duration(seconds: u64) -> u64 {
    seconds
}

pub fn delay(state: State) -> Box<HandlerFuture> {
    let params = DelayParams::borrow_from(&state);
    let delay = min(params.n, 10);

    let f = async move {
        let duration = Duration::from_secs(sleep_duration(delay));
        let _ = Delay::new(duration).await;
        Ok(ok(state, delay.to_string()))
    };
    // let f = Delay::new(Duration::from_secs(sleep_duration(delay))).then(
    //     move |result| {
    //         future_etry!(state, result);
    //         future::ok(ok(state, delay.to_string()))
    //     },
    // );

    Box::new(f.boxed().compat())
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use gotham::test::TestServer;
    use http::StatusCode;

    #[test]
    fn test_sleep() {
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
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
