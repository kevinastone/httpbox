use crate::headers::{Cookie, SetCookie};
use crate::http::{bad_request, ok, response, Request, Result};
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
    let mut res = response();
    for (k, v) in req
        .query::<Vec<(String, String)>>()
        .map_err(|_| bad_request())?
    {
        res = res.typed_header(SetCookie(HTTPCookie::new(k, v)));
    }

    res.into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::HeaderMapExt;
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
}
