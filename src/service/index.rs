use crate::handler::{BoxFuture, Handler};
use crate::http::{Bytes, Request, Result, html};
use crate::router::Route;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: Vec<&'a Route>,
}

#[derive(Debug, Clone)]
pub struct Index(Bytes);

impl Handler for Index {
    fn handle<'a>(&'a self, _: Request) -> BoxFuture<'a, Result> {
        let content = self.0.clone();
        Box::pin(async move { html(content) })
    }
}

pub fn render_index<'a>(routes: impl IntoIterator<Item = &'a Route>) -> String {
    let template = IndexTemplate {
        routes: routes.into_iter().collect(),
    };
    template.render().unwrap()
}

impl<'a, IT: IntoIterator<Item = &'a Route>> From<IT> for Index {
    fn from(routes: IT) -> Self {
        Index(render_index(routes).into())
    }
}
