use std::collections::HashMap;

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
    if let Err(_err) = spotify.request_token(code).await {
        return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16());
    }

    let token = spotify.get_token().lock().await.unwrap().clone().unwrap();

    //sessionにtokenを追加
    let uuid = Uuid::new_v4();
    let kv = ctx.kv("SESSION_KV").unwrap();
    let token_json = serde_json::to_string(&token).unwrap();
    kv.put(uuid.to_string().as_str(), token_json)?
        .expiration_ttl(30000)
        .execute()
        .await?;
    let mut response = Response::ok("Login successful").unwrap();
    response
        .headers_mut()
        .append("Set-Cookie", format!("session_id={};", uuid).as_str())?;
    Ok(response)
}
