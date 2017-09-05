extern crate horrorshow;
extern crate iron;
extern crate router;

use horrorshow::prelude::*;
use self::iron::{Request, Response, IronResult, Handler};
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::router::Router;
use super::routes::{Route, RouteHandler};
use std::convert::From;

pub struct Index(Vec<Route>);

impl Handler for Index {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {

        let body = itry!(html! {
            html {
                head {
                    style {
                        : "
                        ul { list-style-type: none; }
                        code { font-weight: bold; }
                        "
                    }
                }
                body {
                    h1 {
                        : "httpbox: HTTP Testing Service"
                    }
                    h2 {
                        : "Endpoints"
                    }
                    ul {
                        @ for route in self.0.iter() {
                            li {
                                @ if let Some(example_path) = route.example_path() {
                                    a(href=example_path) {
                                        code {
                                            : route.path
                                        }
                                    }
                                } else {
                                    code {
                                        : route.path
                                    }
                                }
                                span { : " - " }
                                span { : route.description }
                            }
                        }
                    }
                }
            }
        }
                                 .into_string());

        Ok(Response::with((status::Ok, Header(headers::ContentType::html()), body)))
    }
}

pub struct IndexBuilder(Vec<RouteHandler>);

impl IndexBuilder {
    pub fn new() -> Self {
        IndexBuilder(vec![])
    }

    pub fn add(&mut self, handler: RouteHandler) -> &mut Self {
        self.0.push(handler);
        self
    }
}

impl From<IndexBuilder> for Router {
    #[inline]
    fn from(source: IndexBuilder) -> Router {
        let mut router = Router::new();

        let mut routes = source
            .0
            .iter()
            .map(|h| h.route.clone())
            .collect::<Vec<Route>>();
        routes.insert(0, Route::new("/").set_description("This page"));

        for handler in source.0 {
            let path = handler.route.path;
            router.route(handler.route.method.clone(),
                         handler.route.path,
                         handler,
                         path);
        }

        let index = Index(routes);
        router.get("/", index, "index");

        router
    }
}
