use crate::handler::Handler;
use crate::http::{
    internal_server_error, not_found, Body, Error, Request, Response,
};
use futures::prelude::*;
use hyper::Request as HTTPRequest;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;
use uri_path::PathMatch;

mod routes;

pub use self::routes::{route, Route};

async fn handle_panics(
    fut: impl Future<Output = crate::http::Result>,
) -> crate::http::Result {
    let wrapped = std::panic::AssertUnwindSafe(fut).catch_unwind();
    wrapped.await.map_err(|_| internal_server_error())?
}

pub struct Endpoint {
    route: Route,
    handler: Box<dyn Handler + Sync>,
}

impl Endpoint {
    fn new<H: Handler + Sync + 'static>(route: Route, handler: H) -> Self {
        Self {
            route,
            handler: Box::new(handler),
        }
    }
}

pub struct RouterBuilder {
    endpoints: Vec<Endpoint>,
}

impl RouterBuilder {
    fn new() -> Self {
        Self { endpoints: vec![] }
    }

    pub fn install<H: Handler + Sync + 'static, R: Into<Route>>(
        mut self,
        handler: H,
        route: R,
    ) -> Self {
        self.endpoints.push(Endpoint::new(route.into(), handler));
        self
    }

    pub fn routes(&self) -> impl Iterator<Item = &Route> {
        self.endpoints.iter().map(|endpoint| &endpoint.route)
    }

    pub fn build(self) -> Router {
        Router::new(RouterInternal {
            endpoints: self.endpoints,
        })
    }
}

pub struct RouterInternal {
    endpoints: Vec<Endpoint>,
}

impl RouterInternal {
    pub fn route<B>(
        &self,
        req: &HTTPRequest<B>,
    ) -> Option<(&Endpoint, PathMatch)> {
        self.endpoints.iter().find_map(|endpoint| {
            endpoint.route.matches(req).map(|params| (endpoint, params))
        })
    }
}

#[derive(Clone)]
pub struct Router(Arc<RouterInternal>);

impl Router {
    fn new(router: RouterInternal) -> Self {
        Router(Arc::new(router))
    }

    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }
}

impl<B: Into<Body> + std::marker::Send + 'static> Service<HTTPRequest<B>>
    for Router
{
    type Response = Response;
    type Error = hyper::http::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HTTPRequest<B>) -> Self::Future {
        let router = self.0.clone();

        async move {
            let (endpoint, matched_path) =
                router.route(&req).ok_or_else(not_found)?;

            let client_req = Request::new(req.map(|b| b.into()), matched_path);
            handle_panics(endpoint.handler.handle(client_req)).await
        }
        .or_else(|e: Error| e.into_result())
        .boxed()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::http::Request;
    use hyper::http::Request as HTTPRequest;
    use hyper::http::StatusCode;

    use uri_path::path;

    #[tokio::test]
    async fn test_panic() {
        let handler = |_: Request| async {
            unimplemented!();
        };

        let router = Router::builder().install(handler, route(path!())).build();
        let mut service = router;

        let body = Body::empty();
        let res = service.call(HTTPRequest::new(body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
