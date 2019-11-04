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

impl NewHandler for Index {
    type Instance = Self;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(self.clone())
    }
}

pub fn render_index(routes: &[Route]) -> String {
    let template = IndexTemplate { routes };
    template.render().unwrap()
}

impl<'a> From<&'a [Route<'a>]> for Index {
    fn from(routes: &'a [Route<'a>]) -> Self {
        Index(render_index(routes))
    }
}
