use worker::{Request,Response,Result,RouteContext};

pub fn handler(req:Request, context :RouteContext<()>) ->Result<Response> {
    Response::ok("this is login page")
}