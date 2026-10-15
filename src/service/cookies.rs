use crate::headers::Cookie;
use crate::http::{Request, Result, bad_request, ok, response};
use cookie::Cookie as HTTPCookie;
use hyper::header::HeaderValue;
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
    let mut res = response();
    for (k, v) in req
        .query::<Vec<(String, String)>>()
        .map_err(|_| bad_request())?
    {
        let cookie = HTTPCookie::new(k, v);
        let header_val = HeaderValue::try_from(cookie.to_string())
            .map_err(|_| bad_request())?;
        res = res.header(hyper::header::SET_COOKIE, header_val);
    }

    res.into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::HeaderMapExt;
    use crate::headers::SetCookie;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_no_cookies() {
        let res = request().handle(cookies).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
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
        let body = res.read_body_utf8().await.unwrap();
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
        let body = res.read_body_utf8().await.unwrap();
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

    #[tokio::test]
    async fn test_set_multiple_cookies() {
        let res = request()
            .path("/?first=value&second=another")
            .handle(set_cookies)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let cookies: Vec<String> = res
            .headers()
            .get_all(hyper::header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(cookies.len(), 2);
        assert!(cookies.contains(&"first=value".to_string()));
        assert!(cookies.contains(&"second=another".to_string()));
    }
}
