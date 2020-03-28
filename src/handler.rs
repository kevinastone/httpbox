use crate::http::{Request, Result};
use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait Handler: Send {
    async fn handle(&self, req: Request) -> Result;
}

#[async_trait]
impl<F, Fut: 'static> Handler for F
where
    Fut: Future<Output = Result> + Send,
    F: Fn(Request) -> Fut + Send + Sync,
{
    #[inline]
    async fn handle(&self, req: Request) -> Result {
        self(req).await
    }
}
