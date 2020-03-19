use futures::prelude::*;
use std::pin::Pin;

pub trait TestResponseExt {
    fn read_utf8_body(
        self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>>>>;
}

impl TestResponseExt for hyper::Response<hyper::Body> {
    fn read_utf8_body(
        self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>>>> {
        async move {
            let bytes = hyper::body::to_bytes(self.into_body()).await?;
            Ok(String::from_utf8(bytes.to_vec())?)
        }
        .boxed()
    }
}
