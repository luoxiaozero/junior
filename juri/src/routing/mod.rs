use crate::{request::HTTPMethod, Request, Response};
mod conversion_router_mod;
mod match_router_mod;

pub use conversion_router_mod::conversion_router;
pub use match_router_mod::{MatchRoute, MatchRouter, match_route, match_route_path};

type HandleFn = fn(request: &Request) -> crate::Result<Response>;
pub type Route = (HTTPMethod, String, HandleFn);

pub struct Router {
    get: Vec<Route>,
    post: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            get: vec![],
            post: vec![],
        }
    }

    pub fn get(&mut self, path: &str, handle: HandleFn) {
        self.get.push((HTTPMethod::GET, path.to_string(), handle));
    }

    pub fn post(&mut self, path: &str, handle: HandleFn) {
        self.post.push((HTTPMethod::GET, path.to_string(), handle));
    }

    pub fn route(&mut self, route: Route) {
        match route.0 {
            HTTPMethod::GET => self.get(&route.1, route.2),
            HTTPMethod::POST => self.post(&route.1, route.2),
        }
    }
}
