use crate::app::response::html;
use crate::router::Route;
use gotham::error::Result;
use gotham::handler::{Handler, HandlerFuture, IntoHandlerFuture, NewHandler};
use gotham::state::State;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use horrorshow::{append_html, html};

#[derive(Debug, Clone)]
pub struct Index(String);

impl Handler for Index {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        html(state, self.0).into_handler_future()
    }
}

pub struct IndexNewHandler(String);

impl NewHandler for IndexNewHandler {
    type Instance = Index;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(Index(self.0.clone()))
    }
}

pub fn render_index(routes: &[Route]) -> String {
    (html! {
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
    })
    .into_string()
    .unwrap()
}

impl<'a> From<&'a [Route<'a>]> for IndexNewHandler {
    fn from(routes: &'a [Route<'a>]) -> Self {
        IndexNewHandler(render_index(routes))
    }
}
