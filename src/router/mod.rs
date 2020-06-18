use crate::handler::Handler;
use crate::http::{internal_server_error, not_found, Error, Request, Response};
use futures::prelude::*;
use hyper::server::conn::AddrStream;
use hyper::{service::Service, Body, Request as HTTPRequest};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

mod routes;

pub use self::routes::{route, Route};
pub use typed_path::{Path, PathSegment};

async fn handle_panics(
    fut: impl Future<Output = crate::http::Result>,
) -> crate::http::Result {
    let wrapped = std::panic::AssertUnwindSafe(fut).catch_unwind();
    wrapped.await.map_err(|_| internal_server_error())?
}

// pub struct Endpoint<T> {
//     route: Route<T>,
//     handler: Box<dyn Handler + Sync>,
// }

// type BoxedResultFuture = Pin<Box<dyn Future<Output = crate::http::Result>>>;

type BoxedChainedResultFuture =
    Pin<Box<dyn Future<Output = Result<crate::http::Result, Request>> + Send>>;

trait Endpoint {
    fn route(&self, req: Request) -> BoxedChainedResultFuture;
}

struct HandlerEndpoint<
    T: Send + for<'a> serde::de::Deserialize<'a> + std::marker::Sync,
    H: Handler<T>,
>(Route<T>, H);

impl<
        T: Send + for<'a> serde::de::Deserialize<'a> + std::marker::Sync + 'static,
        H: Handler<T> + std::marker::Sync + 'static,
    > Endpoint for HandlerEndpoint<T, H>
{
    fn route(&self, req: Request) -> BoxedChainedResultFuture {
        let handle = self.1;
        future::ready(self.0.matches(req.req()).ok_or(req))
            .and_then(|params| handle.handle(req, params).map(Ok))
            .boxed()

        // if let Some(params) = self.0.matches(req.req()) {
        //     async { Ok(self.1.handle(req, params).await) }.boxed()
        // } else {
        //     async { Err(req) }.boxed()
        // }

        // self.0
        //     .matches(req.req())
        //     .map(|params| self.1.handle(req, params).boxed())
        //     .ok_or::<Request>(req)
    }
}

pub struct RouterBuilder {
    endpoints: Vec<Box<dyn Endpoint + Send + Sync>>,
}

impl RouterBuilder {
    fn new() -> Self {
        Self { endpoints: vec![] }
    }

    pub fn install<
        T: 'static + Send + Sync + for<'a> serde::de::Deserialize<'a>,
        H: Handler<T> + Sync + 'static,
        R: Into<Route<T>>,
    >(
        mut self,
        handler: H,
        route: R,
    ) -> Self {
        let route: Route<T> = route.into();

        self.endpoints
            .push(Box::new(HandlerEndpoint(route, handler)));
        self
    }

    // pub fn routes(&self) -> impl Iterator<Item = &Route> {
    //     self.endpoints.iter().map(|endpoint| &endpoint.route)
    // }

    pub fn build(self) -> Router {
        Router::new(RouterInternal {
            endpoints: self.endpoints,
        })
    }
}

pub struct RouterInternal {
    endpoints: Vec<Box<dyn Endpoint + Send + Sync>>,
}

impl RouterInternal {
    pub async fn route(&self, req: Request) -> Option<crate::http::Result> {
        let endpoints = stream::iter(&self.endpoints);
        endpoints
            .fold(Err(req), |result, endpoint| {
                future::ready(result).or_else(move |req| endpoint.route(req))
            })
            .await
            .ok()
    }
}

pub struct Router(Arc<RouterInternal>);

impl Router {
    fn new(router: RouterInternal) -> Self {
        Router(Arc::new(router))
    }

    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }

    pub fn service(&self, addr: Option<SocketAddr>) -> RouterService {
        RouterService::new(&self.0, addr)
    }
}

impl Service<&AddrStream> for Router {
    type Response = RouterService;
    type Error = Infallible;
    type Future = futures::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, conn: &AddrStream) -> Self::Future {
        future::ok(self.service(Some(conn.remote_addr())))
    }
}

pub struct RouterService {
    router: Arc<RouterInternal>,
    client_addr: Option<SocketAddr>,
}

impl RouterService {
    fn new(router: &Arc<RouterInternal>, addr: Option<SocketAddr>) -> Self {
        RouterService {
            router: Arc::clone(router),
            client_addr: addr,
        }
    }
}

impl Service<HTTPRequest<Body>> for RouterService {
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

    fn call(&mut self, req: HTTPRequest<Body>) -> Self::Future {
        let router = self.router.clone();
        let client_addr = self.client_addr;

        let client_req = Request::new(req, client_addr);

        async move {
            router
                .route(client_req)
                .await
                .unwrap_or_else(|| Err(not_found()))

            // handle_panics(router.route(client_req).unwrap_or_else(|| not_found))
            //     .await?
            // let result = router.route(req).await;
            // let (endpoint, matched_path) =
            //     router.route(&req).ok_or_else(not_found)?;
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

    use typed_path::path;

    #[tokio::test]
    async fn test_panic() {
        let handler = |_: Request| async {
            unimplemented!();
        };

        let router = Router::builder().install(handler, route(path!())).build();
        let mut service = router.service(None);

        let res = service.call(HTTPRequest::default()).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
