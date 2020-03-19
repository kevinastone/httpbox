mod uri;

use self::uri::absolute_url;
use crate::http::{bad_request, redirect_to, Request, Result};
use hyper::Uri;
use serde_derive::Deserialize;
use std::cmp::min;

#[derive(Deserialize)]
pub struct RedirectUrlParams {
    url: String,
}

pub async fn to(req: Request) -> Result {
    let query = req
        .query::<RedirectUrlParams>()
        .map_err(|_| bad_request())?;

    let uri = query.url.parse::<Uri>().map_err(|_| bad_request())?;

    redirect_to(uri)
}

pub async fn redirect(req: Request) -> Result {
    relative(req).await
}

pub async fn relative(req: Request) -> Result {
    let n = req.param::<u16>("n").ok_or_else(bad_request)?;
    let n = min(n - 1, 100);

    let url = if n > 0 {
        format!("/relative-redirect/{}", n)
    } else {
        String::from("/")
    };

    let uri = url.parse::<Uri>().map_err(|_| bad_request())?;
    redirect_to(uri)
}

pub async fn absolute(req: Request) -> Result {
    let n = req.param::<u16>("n").ok_or_else(bad_request)?;
    let n = min(n - 1, 100);
    let url = if n > 0 {
        format!("/absolute-redirect/{}", n)
    } else {
        String::from("/")
    };

    let request_uri = req.uri();
    let response_uri = absolute_url(&req, request_uri)
        .and_then(|base| Ok(base.join(&url)?))
        .and_then(|url| Ok(url.to_string().parse::<Uri>()?))
        .map_err(|_| bad_request())?;

    redirect_to(response_uri)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::headers::HeaderMapExt;
    use crate::headers::Host;
    use crate::headers::Location;
    use crate::test::request;
    use hyper::http::StatusCode;
    use hyper::http::{uri::Authority, Uri};

    #[tokio::test]
    async fn test_redirect_to() {
        let res = request()
            .path("/?url=http://example.com")
            .handle(to)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://example.com/")
        )
    }

    #[tokio::test]
    async fn test_redirect() {
        let res = request().param("n", "5").handle(redirect).await.unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/relative-redirect/4")
        )
    }

    #[tokio::test]
    async fn test_redirect_last() {
        let res = request().param("n", "1").handle(redirect).await.unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/")
        )
    }

    #[tokio::test]
    async fn test_relative_redirect() {
        let res = request().param("n", "5").handle(relative).await.unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/relative-redirect/4")
        )
    }

    #[tokio::test]
    async fn test_relative_redirect_last() {
        let res = request().param("n", "1").handle(relative).await.unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/")
        )
    }

    #[tokio::test]
    async fn test_absolute_redirect() {
        let res = request()
            .typed_header(Host::from(Authority::from_static("example.com")))
            .param("n", "5")
            .handle(absolute)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://example.com/absolute-redirect/4")
        )
    }

    #[tokio::test]
    async fn test_absolute_redirect_last() {
        let res = request()
            .typed_header(Host::from(Authority::from_static("example.com")))
            .param("n", "1")
            .handle(absolute)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(
            res.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://example.com/")
        )
    }
}
