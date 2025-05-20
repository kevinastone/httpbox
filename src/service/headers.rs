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
    use hyper::header::HeaderValue;
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
        // Note: Header order is not guaranteed, so check for both lines.
        assert!(body.contains("x-request-id: 1234"));
        assert!(body.contains("user-agent: ExampleBot"));
    }

    #[tokio::test]
    async fn test_headers_with_duplicate_names() {
        let res = request()
            .header("Warning", "199 Miscellaneous warning")
            .header("Warning", "299 Another warning")
            .handle(headers)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        // Note: Order of duplicate headers in the body might not be guaranteed
        assert!(body.contains("warning: 199 Miscellaneous warning"));
        assert!(body.contains("warning: 299 Another warning"));
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

    #[tokio::test]
    async fn test_response_headers_with_duplicate_names() {
        let res = request()
            .path("/?Warning=199%20Miscellaneous%20warning&Warning=299%20Another%20warning")
            .handle(response_headers)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let warnings: Vec<&HeaderValue> = res.headers().get_all("Warning").iter().collect();
        assert_eq!(warnings.len(), 2);
        assert!(warnings.contains(&&HeaderValue::from_static("199 Miscellaneous warning")));
        assert!(warnings.contains(&&HeaderValue::from_static("299 Another warning")));
    }
}
