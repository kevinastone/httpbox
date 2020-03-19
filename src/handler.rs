use crate::http::{Request, Result};
use std::future::Future;
use std::pin::Pin;

pub type HandlerFuture = dyn Future<Output = Result> + Send;

pub trait Handler: Send {
    fn handle(&self, req: Request) -> Pin<Box<HandlerFuture>>;
}

impl<F, Fut: 'static> Handler for F
where
    Fut: Future<Output = Result> + Send,
    F: Fn(Request) -> Fut + Send + Sync,
{
    #[inline]
    fn handle(&self, req: Request) -> Pin<Box<HandlerFuture>> {
        Box::pin(self(req))
    }
}
