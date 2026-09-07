use crate::headers::{CacheControl, ContentType};
use crate::http::{Request, Result, response};
use std::time::Duration;

const FAVICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/favicon.svg"
));

pub async fn favicon(_req: Request) -> Result {
    response()
        .typed_header(ContentType::from(mime::IMAGE_SVG))
        .typed_header(
            CacheControl::new().with_max_age(Duration::from_secs(86400)),
        )
        .body(FAVICON_BYTES)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_favicon() {
        let res = request().handle(favicon).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "image/svg+xml");
        let body = res.read_body().await.unwrap();
        assert_eq!(body, FAVICON_BYTES);
    }
}
