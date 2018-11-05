extern crate gotham;

use super::FrozenRoute;

use gotham::extractor::{PathExtractor, QueryStringExtractor};
use gotham::handler::Handler;
use gotham::pipeline::chain::PipelineHandleChain;
use gotham::router::builder::*;
use std::convert::Into;
use std::panic::RefUnwindSafe;

pub struct RouteInstaller<'a, 'b: 'a, C, P>
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: Send + Sync + RefUnwindSafe + 'static,
{
    builder: &'a mut RouterBuilder<'b, C, P>,
    routes: Vec<FrozenRoute<'a>>,
}

impl<'a, 'b, C, P> RouteInstaller<'a, 'b, C, P>
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: Send + Sync + RefUnwindSafe + 'static,
{
    pub fn new(builder: &'a mut RouterBuilder<'b, C, P>) -> Self {
        RouteInstaller {
            builder: builder,
            routes: vec![],
        }
    }

    pub fn install<H, R>(&mut self, handler: H, route: R) -> FrozenRoute<'a>
    where
        H: Handler + RefUnwindSafe + Send + Sync + Copy + 'static,
        R: Into<FrozenRoute<'a>>,
    {
        let route: FrozenRoute = route.into();
        self.routes.push(route.clone());
        let method = route.method();
        self.builder.associate(route.path(), move |assoc| {
            assoc.request(vec![method]).to(handler)
        });
        route
    }

    pub fn routes(&self) -> Vec<FrozenRoute<'a>> {
        self.routes.clone()
    }

    pub fn install_with_path_extractor<H, R, PE>(
        &mut self,
        handler: H,
        route: R,
    ) -> FrozenRoute<'a>
    where
        H: Handler + RefUnwindSafe + Send + Sync + Copy + 'static,
        R: Into<FrozenRoute<'a>>,
        PE: PathExtractor + Send + Sync + 'static,
    {
        let route: FrozenRoute = route.into();
        self.routes.push(route.clone());
        let method = route.method();
        self.builder.associate(route.path(), move |assoc| {
            assoc
                .request(vec![method])
                .with_path_extractor::<PE>()
                .to(handler)
        });
        route
    }

    pub fn install_with_query_extractor<H, R, QSE>(
        &mut self,
        handler: H,
        route: R,
    ) -> FrozenRoute<'a>
    where
        H: Handler + RefUnwindSafe + Send + Sync + Copy + 'static,
        R: Into<FrozenRoute<'a>>,
        QSE: QueryStringExtractor + Send + Sync + 'static,
    {
        let route: FrozenRoute = route.into();
        self.routes.push(route.clone());
        let method = route.method();
        self.builder.associate(route.path(), move |assoc| {
            assoc
                .request(vec![method])
                .with_query_string_extractor::<QSE>()
                .to(handler)
        });
        route
    }

    pub fn install_with_path_and_query_extractor<H, R, PE, QSE>(
        &mut self,
        handler: H,
        route: R,
    ) -> FrozenRoute<'a>
    where
        H: Handler + RefUnwindSafe + Send + Sync + Copy + 'static,
        R: Into<FrozenRoute<'a>>,
        PE: PathExtractor + Send + Sync + 'static,
        QSE: QueryStringExtractor + Send + Sync + 'static,
    {
        let route: FrozenRoute = route.into();
        self.routes.push(route.clone());
        let method = route.method();
        self.builder.associate(route.path(), move |assoc| {
            assoc
                .request(vec![method])
                .with_path_extractor::<PE>()
                .with_query_string_extractor::<QSE>()
                .to(handler)
        });
        route
    }

    pub fn closure<R, F>(&mut self, route: R, closure: F) -> FrozenRoute<'a>
    where
        F: FnOnce(&FrozenRoute<'a>, &mut RouterBuilder<C, P>),
        R: Into<FrozenRoute<'a>>,
    {
        let route: FrozenRoute = route.into();
        self.routes.push(route.clone());
        closure(&route, self.builder);
        route
    }
}
