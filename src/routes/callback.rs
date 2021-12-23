use std::{collections::HashMap};

use http::StatusCode;
use rspotify::clients::{BaseClient, OAuthClient};
use uuid::Uuid;
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    //getパラメーターからcodeを取得
    let query_pairs: HashMap<_, _> = req.url()?.query_pairs().into_owned().collect();
    let code = match query_pairs.get("code") {
        Some(token) => token,
        None => return Response::error("NoAuthCode", StatusCode::BAD_REQUEST.as_u16()),
    };

    //Spotifyクライアントを作成しトークンを取得
    let mut spotify = match utils::get_spotify_client(&ctx) {
        Ok(spotify) => spotify,
        Err(err) => {
            return Response::error(
                format!("Err:INTERNAL_SERVER_ERROR \n {:?}", err),
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            )
        }
    };
    if let Err(err) = spotify.request_token(code).await {
        return Response::error(format!("UNAUTHORIZED \n {:?}",err), StatusCode::UNAUTHORIZED.as_u16());
    }

    let token = match spotify.get_token().lock().await{
        Ok(mutex_token) => mutex_token.clone(),
        Err(err) => return Response::error(format!("INTERNAL_SERVER_ERROR : Can't lock memory \n {:?}",err), StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
    };

    let token = match token{
        Some(token) => token,
        None => return Response::error("INTERNAL_SERVER_ERROR : can't get token", StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
    };

    let session_ttl : u64 = match ctx.var("SESSION_TTL"){
        Ok(session_ttl) => session_ttl.to_string().parse().unwrap(),
        Err(_) => return Response::error("INTERNAL_SERVER_ERROR : can't get token", StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
    };

    //sessionにtokenを追加
    let uuid = Uuid::new_v4();
    let kv = match ctx.kv("SESSION_KV"){
        Ok(kv) => kv,
        Err(err) => return Response::error(format!("INTERNAL_SERVER_ERROR : Can't get KvStore \n {:?}",err), StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
    };
    let token_json = match  serde_json::to_string(&token){
        Ok(token_json) => token_json,
        Err(err) => return Response::error(format!("INTERNAL_SERVER_ERROR : Can't parse Json \n {:?}",err), StatusCode::INTERNAL_SERVER_ERROR.as_u16())
    };
    kv.put(uuid.to_string().as_str(), token_json)?
        .expiration_ttl(session_ttl)
        .execute()
        .await?;
    let mut response = Response::ok("Login successful").unwrap();
    response
        .headers_mut()
        .append("Set-Cookie", format!("session_id={}; Max-Age={}; Secure; SameSite=None;", uuid,session_ttl).as_str())?;
    Ok(response)
}
