use crate::handler::Handler;
use crate::http::{Bytes, Request, Result, html};
use crate::router::Route;
use askama::Template;
use async_trait::async_trait;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: Vec<&'a Route>,
}

#[derive(Debug, Clone)]
pub struct Index(Bytes);

#[async_trait]
impl Handler for Index {
    async fn handle(&self, _: Request) -> Result {
        html(self.0.clone())
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
