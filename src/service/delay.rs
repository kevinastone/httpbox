use crate::http::{Request, Result, bad_request, ok};
use std::cmp::min;
use std::time::Duration;
use tokio::time::sleep;

pub async fn delay(req: Request) -> Result {
    let n = req.param::<u64>("n").ok_or_else(bad_request)?;
    let delay = min(n, 10);

    sleep(Duration::from_secs(delay)).await;
    ok(delay.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    // With `start_paused = true`, Tokio's mock clock pauses wall-clock time and
    // automatically jumps virtual time forward to the next scheduled deadline whenever
    // the runtime is otherwise idle and waiting on a timer. This executes `sleep`
    // instantly without real-time delay while still verifying the requested duration.
    #[tokio::test(start_paused = true)]
    async fn test_sleep() {
        let start = tokio::time::Instant::now();
        let res = request().param("n", "3").handle(delay).await.unwrap();

        assert_eq!(tokio::time::Instant::now() - start, Duration::from_secs(3));
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "3");
    }

    #[tokio::test(start_paused = true)]
    async fn test_sleep_too_long() {
        let start = tokio::time::Instant::now();
        let res = request().param("n", "33").handle(delay).await.unwrap();

        assert_eq!(
            tokio::time::Instant::now() - start,
            Duration::from_secs(10)
        );
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
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
