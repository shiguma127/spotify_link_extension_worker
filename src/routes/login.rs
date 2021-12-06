use worker::{Request,Response,Result,RouteContext, Url};

pub fn handler(_req:Request, _context:RouteContext<()>) ->Result<Response> {
    Response::redirect_with_status(Url::parse("http://localhost:8787/").unwrap(), 301)
}