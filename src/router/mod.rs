use crate::handler::Handler;
use crate::http::{
    Body, Error, Request, Response, internal_server_error, not_found,
};
use futures::prelude::*;
use hyper::Request as HTTPRequest;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;
use uri_path::PathMatch;

mod routes;

pub use self::routes::{Route, route};

type BoxHandler = Box<dyn Handler + Sync>;

#[derive(Default)]
struct MethodMap<T> {
    entries: Vec<(hyper::Method, T)>,
}

impl<T> MethodMap<T> {
    fn get(&self, method: &hyper::Method) -> Option<&T> {
        self.entries
            .iter()
            .find(|(m, _)| m == method)
            .map(|(_, val)| val)
    }

    fn entry(&mut self, method: hyper::Method) -> Entry<'_, T> {
        if let Some(pos) = self.entries.iter().position(|(m, _)| *m == method) {
            Entry::Occupied(&mut self.entries[pos].1)
        } else {
            Entry::Vacant(self, method)
        }
    }
}

enum Entry<'a, T> {
    Occupied(&'a mut T),
    Vacant(&'a mut MethodMap<T>, hyper::Method),
}

impl<'a, T> Entry<'a, T> {
    fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Occupied(val) => val,
            Entry::Vacant(map, method) => {
                map.entries.push((method, default));
                &mut map.entries.last_mut().unwrap().1
            }
        }
    }

    fn or_default(self) -> &'a mut T
    where
        T: Default,
    {
        self.or_insert(T::default())
    }
}

async fn handle_panics(
    fut: impl Future<Output = crate::http::Result>,
) -> crate::http::Result {
    let wrapped = std::panic::AssertUnwindSafe(fut).catch_unwind();
    wrapped.await.map_err(|_| internal_server_error())?
}

pub struct Endpoint {
    route: Route,
    handler: BoxHandler,
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
        let mut routes_by_method: MethodMap<uri_path::Router<BoxHandler>> =
            MethodMap::default();

        for endpoint in self.endpoints {
            let (method, path) = endpoint.route.into_parts();
            routes_by_method
                .entry(method)
                .or_default()
                .insert(path, endpoint.handler);
        }

        Router::new(RouterInternal { routes_by_method })
    }
}

struct RouterInternal {
    routes_by_method: MethodMap<uri_path::Router<BoxHandler>>,
}

impl RouterInternal {
    fn route<B>(
        &self,
        req: &HTTPRequest<B>,
    ) -> Option<(&BoxHandler, PathMatch)> {
        let router = self.routes_by_method.get(req.method())?;
        let matched = router.at(req.uri().path())?;
        Some((matched.value, matched.params))
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

impl<B: Into<Body> + Send + 'static> Service<HTTPRequest<B>> for Router {
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
            let (handler, matched_path) =
                router.route(&req).ok_or_else(not_found)?;

            let client_req = Request::new(req.map(|b| b.into()), matched_path);
            handle_panics(handler.handle(client_req)).await
        }
        .or_else(|e: Error| e.into_result())
        .boxed()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::http::Request;
    use crate::test::TestResponseExt;
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

    #[tokio::test]
    async fn test_catch_all_route() {
        let handler = |req: Request| async move {
            let path = req.param::<String>("path").unwrap();
            crate::http::response().body(path)
        };

        let router = Router::builder()
            .install(handler, route(path!("static" / *path)))
            .build();
        let mut service = router;

        let req = HTTPRequest::builder()
            .uri("/static/css/theme/dark.css")
            .body(Body::empty())
            .unwrap();

        let res = service.call(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body().await.unwrap();
        assert_eq!(body, "css/theme/dark.css".as_bytes());
    }
}
