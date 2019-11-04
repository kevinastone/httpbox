use crate::app::response::ok;
use futures_timer::Delay;
use gotham::state::{FromState, State};
use gotham_async::async_handler;
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

#[async_handler]
pub async fn delay(state: State) -> (State, Response) {
    let params = DelayParams::borrow_from(&state);
    let delay = min(params.n, 10);

    let duration = Duration::from_secs(sleep_duration(delay));
    let _ = Delay::new(duration).await;
    ok(state, delay.to_string())
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
