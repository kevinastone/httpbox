use crate::handler::Handler;
use crate::http::{html, Request, Result};
use crate::router::Route;
use askama::Template;
use async_trait::async_trait;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: Vec<&'a Route>,
}

#[derive(Debug, Clone)]
pub struct Index(String);

#[async_trait]
impl Handler for Index {
    async fn handle(&self, _: Request) -> Result {
        html(self.0.clone())
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
