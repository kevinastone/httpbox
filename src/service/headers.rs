use crate::http::{bad_request, ok, response, Request, Result};
use hyper::header::{HeaderName, HeaderValue};
use itertools::{process_results, Itertools};

pub async fn headers(req: Request) -> Result {
    let request_headers = req
        .headers()
        .iter()
        .map(|(n, v)| v.to_str().map(|v| (n, v)));

    ok(process_results(request_headers, |iter| {
        iter.format_with("\n", |(n, v), f| {
            f(&format_args!("{}: {}", n, v.trim()))
        })
        .to_string()
    })
    .map_err(|_| bad_request())?)
}

pub async fn response_headers(req: Request) -> Result {
    let output_headers = req
        .query::<Vec<(String, String)>>()
        .map_err(|_| bad_request())?
        .iter()
        .map(|(name, value)| {
            Ok((name.parse::<HeaderName>()?, value.parse::<HeaderValue>()?))
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .map_err(|_| bad_request())?;

    let mut res = response();
    for (key, value) in output_headers {
        res = res.header(key, value);
    }
    res.into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_headers() {
        let res = request()
            .header("X-Request-ID", "1234")
            .header("User-Agent", "ExampleBot")
            .handle(headers)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "x-request-id: 1234\nuser-agent: ExampleBot")
    }

    #[tokio::test]
    async fn test_response_headers() {
        let res = request()
            .path("/?X-Request-ID=1234")
            .handle(response_headers)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("X-Request-ID").unwrap(), "1234")
    }
}
