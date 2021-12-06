use std::{collections::HashMap};

use http::StatusCode;
use uuid::Uuid;
use worker::{Request,Response,Result,RouteContext};

pub async fn handler(req:Request, ctx :RouteContext<()>) ->Result<Response> {
    let query_pairs:HashMap<_,_> = req.url()?.query_pairs().into_owned().collect();
    let token = match query_pairs.get("token"){
        Some(token) => token,
        None => return Response::error("BadRequest", StatusCode::BAD_REQUEST.as_u16())
    };
    let uuid = Uuid::new_v4();
    let kv = ctx.kv("SESSION_KV").unwrap();
    kv.put(uuid.to_string().as_str(), token)?.expiration_ttl(60).execute().await?;
    let mut response = Response::ok("uuid").unwrap();
    response.headers_mut().append("Set-Cookie", format!("session_id={}", "session_id").as_str())?;
    Ok(response)
}
