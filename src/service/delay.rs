use crate::http::{bad_request, ok, Request, Result};
use futures_timer::Delay;
use std::cmp::min;
use std::time::Duration;

#[cfg(test)]
fn sleep_duration(_: u64) -> u64 {
    0
}

#[cfg(not(test))]
#[inline]
fn sleep_duration(seconds: u64) -> u64 {
    seconds
}

pub async fn delay(req: Request) -> Result {
    let n = req.param::<u64>("n").ok_or_else(bad_request)?;
    let delay = min(n, 10);

    let duration = Duration::from_secs(sleep_duration(delay));
    let _ = Delay::new(duration).await;
    ok(delay.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_sleep() {
        let res = request().param("n", "3").handle(delay).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "3");
    }

    #[tokio::test]
    async fn test_sleep_too_long() {
        let res = request().param("n", "33").handle(delay).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "10");
    }

    #[tokio::test]
    async fn test_invalid_param() {
        let res = request().param("n", "abc").handle(delay).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_missing_param() {
        let res = request().handle(delay).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
