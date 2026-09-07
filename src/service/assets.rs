use crate::headers::{CacheControl, ContentType};
use crate::http::{Request, Result, not_found, response};
use rust_embed::Embed;
use std::time::Duration;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Assets;

pub async fn assets(req: Request) -> Result {
    let path = req.param::<String>("path").ok_or_else(not_found)?;
    let file = Assets::get(&path).ok_or_else(not_found)?;

    let mime = file
        .metadata
        .mimetype()
        .parse::<mime::Mime>()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    response()
        .typed_header(ContentType::from(mime))
        .typed_header(
            CacheControl::new().with_max_age(Duration::from_secs(86400)),
        )
        .body(file.data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_assets_favicon() {
        let res = request()
            .param("path", "favicon.svg")
            .handle(assets)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "image/svg+xml");
        let body = res.read_body().await.unwrap();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn test_assets_not_found() {
        let res = request()
            .param("path", "nonexistent.png")
            .handle(assets)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
