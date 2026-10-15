use crate::headers::ContentLength;
use crate::headers::ContentType;
use crate::http::{Request, Result, bad_request, body_from_stream, response};
use crate::random::rng;
use futures::prelude::*;
use rand::Rng;
use serde::Deserialize;
use std::cmp::min;
use std::iter::ExactSizeIterator;
use std::num::NonZeroUsize;

pub const MAX_BYTES: u32 = 100 * 1024;

#[derive(Deserialize)]
pub struct BytesQueryParams {
    seed: Option<u32>,
    chunk_size: Option<NonZeroUsize>,
}

pub fn iter_bytes(
    count: u32,
    seed: Option<u32>,
) -> impl ExactSizeIterator<Item = u8> {
    let mut rng = rng(seed);
    (0..count).map(move |_| rng.random::<u8>())
}

pub async fn bytes(req: Request) -> Result {
    let n = req.param::<u32>("n").ok_or_else(bad_request)?;
    let n = min(n, MAX_BYTES);
    let query = req.query::<BytesQueryParams>().map_err(|_| bad_request())?;

    let data = iter_bytes(n, query.seed).collect::<Vec<u8>>();

    response()
        .typed_header(ContentType::octet_stream())
        .body(data)
}

pub async fn stream_bytes(req: Request) -> Result {
    let n = req.param::<u32>("n").ok_or_else(bad_request)?;
    let n = min(n, MAX_BYTES);
    let query = req.query::<BytesQueryParams>().map_err(|_| bad_request())?;

    let data = iter_bytes(n, query.seed);
    let chunk_size = query.chunk_size.map(NonZeroUsize::get).unwrap_or(1);
    let content_length = data.len() as u64;

    response()
        .typed_header(ContentType::octet_stream())
        .typed_header(ContentLength(content_length))
        .body(body_from_stream(stream::iter(data).chunks(chunk_size)))
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_bytes() {
        let res = request()
            .param("n", "4")
            .path("/?seed=1234")
            .handle(bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.read_body().await.unwrap(), [214, 212, 32, 32])
    }

    #[tokio::test]
    async fn test_bytes_with_bad_seed() {
        let res = request()
            .param("n", "4")
            .path("/?seed=abc")
            .handle(bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_stream_bytes() {
        let res = request()
            .param("n", "4")
            .path("/?seed=1234")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.read_body().await.unwrap(), [214, 212, 32, 32])
    }

    #[tokio::test]
    async fn test_stream_bytes_with_bad_seed() {
        let res = request()
            .param("n", "4")
            .path("/?seed=abc")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_stream_bytes_with_chunk_size() {
        let res = request()
            .param("n", "4")
            .path("/?seed=1234&chunk_size=2")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.read_body().await.unwrap(), [214, 212, 32, 32])
    }

    #[tokio::test]
    async fn test_stream_bytes_with_bad_chunk_size() {
        let res = request()
            .param("n", "4")
            .path("/?seed=1234&chunk_size=abc")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_stream_bytes_with_zero_chunk_size() {
        let res = request()
            .param("n", "4")
            .path("/?seed=1234&chunk_size=0")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_bytes_capped_at_max() {
        let res = request().param("n", "200000").handle(bytes).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.read_body().await.unwrap().len(), MAX_BYTES as usize);
    }

    #[tokio::test]
    async fn test_stream_bytes_capped_at_max() {
        let res = request()
            .param("n", "200000")
            .path("/?chunk_size=1024")
            .handle(stream_bytes)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.read_body().await.unwrap().len(), MAX_BYTES as usize);
    }
}
