use crate::headers::{CacheControl, IfModifiedSince, IfNoneMatch};
use crate::http::{bad_request, ok, response, Request, Result, StatusCode};
use std::time::Duration;

pub async fn cache(req: Request) -> Result {
    if req.typed_header::<IfModifiedSince>().is_some()
        || req.typed_header::<IfNoneMatch>().is_some()
    {
        response().status(StatusCode::NOT_MODIFIED).into()
    } else {
        ok("")
    }
}

pub async fn set_cache(req: Request) -> Result {
    let n = req.param::<u64>("n").ok_or_else(bad_request)?;

    response()
        .typed_header(CacheControl::new().with_max_age(Duration::from_secs(n)))
        .into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::HeaderMapExt;
    use crate::test::*;
    use hyper::http::StatusCode;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_cache_no_headers() {
        let res = request().handle(cache).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cache_if_modified_since() {
        let header: IfModifiedSince = SystemTime::now().into();
        let res = request().typed_header(header).handle(cache).await.unwrap();

        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn test_cache_if_none_match() {
        let res = request()
            .typed_header(IfNoneMatch::any())
            .handle(cache)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn test_set_cache() {
        let res = request().param("n", "30").handle(set_cache).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().typed_get::<CacheControl>().unwrap(),
            CacheControl::new().with_max_age(Duration::from_secs(30))
        )
    }
}
