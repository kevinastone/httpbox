use crate::headers::{Cookie, SetCookie};
use crate::http::{
    bad_request, ok, Body, HTTPResponse, Request, ResponseTypedHeaderExt,
    Result,
};
use cookie::Cookie as HTTPCookie;
use itertools::Itertools;

pub async fn cookies(req: Request) -> Result {
    let cookies = req.typed_header::<Cookie>();

    let body = cookies
        .iter()
        .flat_map(|cookie| cookie.iter())
        .format_with("\n", |cookie, f| {
            f(&format_args!("{} = {}", cookie.name(), cookie.value()))
        })
        .to_string();

    ok(body)
}

pub async fn set_cookies(req: Request) -> Result {
    let response_cookies = req
        .query::<Vec<(String, String)>>()
        .map_err(|_| bad_request())?
        .into_iter()
        .map(|(k, v)| SetCookie(HTTPCookie::new(k, v)));

    let mut res = HTTPResponse::builder();
    for cookie in response_cookies {
        res = res.typed_header(cookie);
    }

    res.body(Body::empty()).map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::HeaderMapExt;
    use crate::test::{request, TestResponseExt};
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_no_cookies() {
        let res = request().handle(cookies).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "");
    }

    #[tokio::test]
    async fn test_cookies() {
        let res = request()
            .typed_header(Cookie(vec![HTTPCookie::new("test", "value")]))
            .handle(cookies)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "test = value");
    }

    #[tokio::test]
    async fn test_multiple_cookies() {
        let res = request()
            .typed_header(Cookie(vec![
                HTTPCookie::new("first", "value"),
                HTTPCookie::new("second", "another"),
            ]))
            .handle(cookies)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "first = value\nsecond = another");
    }

    #[tokio::test]
    async fn test_set_cookies() {
        let res = request()
            .path("/?test=value")
            .handle(set_cookies)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().typed_get::<SetCookie>().unwrap(),
            SetCookie(HTTPCookie::new("test", "value"))
        )
    }
}
