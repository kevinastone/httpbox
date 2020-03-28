use async_trait::async_trait;

#[async_trait]
pub trait TestResponseExt: Sized {
    async fn read_body(self) -> anyhow::Result<Vec<u8>>;
    async fn read_body_utf8(self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.read_body().await?)?)
    }
}

#[async_trait]
impl TestResponseExt for hyper::Response<hyper::Body> {
    async fn read_body(self) -> anyhow::Result<Vec<u8>> {
        let bytes = hyper::body::to_bytes(self.into_body()).await?;
        Ok(bytes.to_vec())
    }
}
