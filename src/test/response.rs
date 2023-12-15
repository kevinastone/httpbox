use crate::http::Body;
use async_trait::async_trait;
use http_body_util::BodyExt;

#[async_trait]
pub trait TestResponseExt: Sized {
    async fn read_body(self) -> anyhow::Result<Vec<u8>>;
    async fn read_body_utf8(self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.read_body().await?)?)
    }
}

#[async_trait]
impl TestResponseExt for hyper::Response<Body> {
    async fn read_body(self) -> anyhow::Result<Vec<u8>> {
        let bytes = BodyExt::collect(self.into_body()).await?.to_bytes();
        Ok(bytes.to_vec())
    }
}
