use crate::handler::Handler;
use crate::http::{not_found, Error, Request, Response};
use crate::path::MatchedPath;
use futures::prelude::*;
use hyper::{service::Service, Body, Request as HTTPRequest};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

mod routes;

pub use self::routes::{route, Route};
pub use crate::path::{Path, PathSegment};

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

    pub fn routes(&self) -> Vec<&Route> {
        self.endpoints
            .iter()
            .map(|endpoint| &endpoint.route)
            .collect::<Vec<&Route>>()
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
    pub fn route(
        &self,
        req: &HTTPRequest<Body>,
    ) -> Option<(&Endpoint, MatchedPath)> {
        self.endpoints.iter().find_map(|endpoint| {
            endpoint
                .route
                .matches(req)
                .map(|matched_path| (endpoint, matched_path))
        })
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
        RouterService::new(&Arc::clone(&self.0), addr)
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

        async move {
            let (endpoint, matched_path) =
                router.route(&req).ok_or_else(not_found)?;

            let client_req = Request::new(req, client_addr, Some(matched_path));
            Ok(endpoint.handler.handle(client_req).await?)
        }
        .or_else(|e: Error| e.into_result())
        .boxed()
    }
}
