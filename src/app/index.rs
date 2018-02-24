extern crate gotham;
extern crate horrorshow;
extern crate hyper;
extern crate mime;

use app::response::ok;
use gotham::handler::{Handler, HandlerFuture, IntoHandlerFuture, NewHandler};
use gotham::state::State;
use horrorshow::prelude::*;
use horrorshow::helper::doctype;
use std::io;
use app::router::FrozenRoute;

#[derive(Debug, Clone)]
pub struct Index(String);

impl Handler for Index {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        ok(state, self.0.into_bytes()).into_handler_future()
    }
}

pub struct IndexNewHandler(String);

impl NewHandler for IndexNewHandler {
    type Instance = Index;

    fn new_handler(&self) -> io::Result<Self::Instance> {
        Ok(Index(self.0.clone()))
    }
}

pub fn render_index(routes: &[FrozenRoute]) -> String {
    let body = html! {
        : doctype::HTML;
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
                    @ for route in routes {
                        li {
                            @ if let Some(example_path) = route.example_path() {
                                a(href=example_path) {
                                    code {
                                        : route.path()
                                    }
                                }
                            } else {
                                code {
                                    : route.path()
                                }
                            }
                            span { : " - " }
                            span { : route.description() }
                        }
                    }
                }
            }
        }
    }.into_string()
        .unwrap();
    body
}

impl<'a> From<&'a [FrozenRoute<'a>]> for IndexNewHandler {
    fn from(routes: &'a [FrozenRoute<'a>]) -> Self {
        IndexNewHandler(render_index(routes))
    }
}
