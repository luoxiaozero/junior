use std::sync::Arc;
use crate::{Response, Request};

pub type Route = (String, fn(request: Request) -> Response);
#[derive(Clone)]
pub struct Router {
    pub get: Vec<Route>,
    pub post: Vec<Route>,
}

pub fn handle_router(request: &Request, router: Arc<Router>) -> Option<fn(Request) -> Response> {
    if request.method == "GET" {
        for route in &router.get {
            if match_router_path(route.0.clone(), request.path.clone()) {
                return Some(route.1);
            }
        }
    } else if request.method == "POST" {
        for route in &router.post {
            if match_router_path(route.0.clone(), request.path.clone()) {
                return Some(route.1);
            }
        }
    }
    None
}

fn match_router_path(route_path: String, path: String) -> bool {
    return route_path == path;
}
