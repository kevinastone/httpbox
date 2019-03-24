use crate::app::response::html;
use crate::router::Route;
use askama::Template;
use gotham::error::Result;
use gotham::handler::{Handler, HandlerFuture, IntoHandlerFuture, NewHandler};
use gotham::state::State;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    routes: &'a [Route<'a>],
}

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
    let template = IndexTemplate { routes };
    template.render().unwrap()
}

impl<'a> From<&'a [Route<'a>]> for IndexNewHandler {
    fn from(routes: &'a [Route<'a>]) -> Self {
        IndexNewHandler(render_index(routes))
    }
}
