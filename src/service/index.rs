use crate::handler::{Handler, HandlerFuture};
use crate::http::{html, Request};
use crate::router::Route;
use askama::Template;
use futures::prelude::*;
use std::pin::Pin;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: Vec<&'a Route>,
}

#[derive(Debug, Clone)]
pub struct Index(String);

impl Handler for Index {
    fn handle(&self, _: Request) -> Pin<Box<HandlerFuture>> {
        future::ready(html(self.0.clone())).boxed()
    }
}

pub fn render_index(routes: Vec<&'_ Route>) -> String {
    let template = IndexTemplate { routes };
    template.render().unwrap()
}

impl<'a> From<Vec<&'a Route>> for Index {
    fn from(routes: Vec<&'a Route>) -> Self {
        Index(render_index(routes))
    }
}
