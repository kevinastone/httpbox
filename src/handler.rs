use crate::http::{Request, Result};
use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait Handler<T: Send>: Send {
    async fn handle(&self, req: Request, params: T) -> Result;
}

#[async_trait]
impl<T: Send + 'static, F, Fut: 'static> Handler<T> for F
where
    Fut: Future<Output = Result> + Send,
    F: Fn(T, Request) -> Fut + Send + Sync,
{
    #[inline]
    async fn handle(&self, req: Request, params: T) -> Result {
        self(params, req).await
    }
}

// #[async_trait]
// impl<F, Fut: 'static> Handler<()> for F
// where
//     Fut: Future<Output = Result> + Send,
//     F: Fn(Request) -> Fut + Send + Sync,
// {
//     #[inline]
//     async fn handle(&self, req: Request, params: ()) -> Result {
//         self(req).await
//     }
// }
