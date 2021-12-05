use std::collections::HashMap;

use http::StatusCode;
use worker::{Request,Response,Result,RouteContext};

pub fn handler(req:Request, _ :RouteContext<()>) ->Result<Response> {
    let query_pairs:HashMap<_,_> = req.url()?.query_pairs().into_owned().collect();
    let token = match query_pairs.get("token"){
        Some(token) => token,
        None => return Response::error("BadRequest", StatusCode::BAD_REQUEST.as_u16())
    };
    Response::ok(format!("{}",token))
}