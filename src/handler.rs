use crate::http::{Request, Result};
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Handler: Send {
    fn handle<'a>(&'a self, req: Request) -> BoxFuture<'a, Result>;
}

impl<F, Fut: 'static> Handler for F
where
    Fut: Future<Output = Result> + Send,
    F: Fn(Request) -> Fut + Send + Sync,
{
    #[inline]
    fn handle<'a>(&'a self, req: Request) -> BoxFuture<'a, Result> {
        Box::pin(self(req))
    }
}
