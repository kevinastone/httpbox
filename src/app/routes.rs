#![allow(dead_code)]

extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult, Handler};
use self::router::Router;


pub struct Routes {
    router: Router,
}

impl Handler for Routes {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        self.router.handle(req)
    }
}

impl Routes {
    pub fn new() -> Self {
        Routes { router: Router::new() }
    }

    pub fn get<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.get(glob, handler);
        self
    }

    pub fn post<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.post(glob, handler);
        self
    }

    pub fn put<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.put(glob, handler);
        self
    }

    pub fn delete<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.delete(glob, handler);
        self
    }

    pub fn patch<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.patch(glob, handler);
        self
    }

    pub fn options<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.options(glob, handler);
        self
    }

    pub fn any<H: Handler, S: AsRef<str>>(&mut self, glob: S, handler: H) -> &Self {
        self.router.any(glob, handler);
        self
    }

    pub fn to_router(self) -> Router {
        self.router
    }
}
