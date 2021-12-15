use http::StatusCode;
use rspotify::{
    clients::OAuthClient,
    model::{AdditionalType, PlayableItem},
    AuthCodeSpotify, Token,
};
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
     let cookie_string = match req.headers().get("cookie") {
        Ok(cookie) => cookie,
        Err(err) => return Response::error(format!("UNAUTHORIZED : can't get cookie header \n {:?}",err), StatusCode::UNAUTHORIZED.as_u16()),
    };
    let cookie_string = match cookie_string {
        Some(cookie) => cookie,
        None => return Response::error("UNAUTHORIZED : cookie is None ", StatusCode::UNAUTHORIZED.as_u16()),
    };
    let cookie = utils::get_cookie_from_string(cookie_string);
    let session_id = match cookie.get("session_id"){
        Some(session_id) => session_id,
        None => return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16()),
    };

    let kv = match ctx.kv("SESSION_KV"){
        Ok(kv) => kv,
        Err(err) => return Response::error(format!("INTERNAL_SERVER_ERROR : Can't get KvStore \n {:?}",err), StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
    };

    let token_json = match kv.get(session_id).await {
        Ok(token_json) => token_json,
        Err(err) => return Response::error(format!("UNAUTHORIZED : can't get session value from KvStore \n {:?}",err), StatusCode::UNAUTHORIZED.as_u16()),
    };

    let token_json = match token_json {
        Some(token_json) => token_json.as_string(),
        None => return Response::error("UNAUTHORIZED : there is no session", StatusCode::UNAUTHORIZED.as_u16()),
    };

    let token = match serde_json::from_str::<Token>(&token_json) {
        Ok(token) => token,
        Err(err) => {
            return Response::error(
                format!("INTERNAL_SERVER_ERROR \n {:?}",err),
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            )
        }
    };

    let mut spotify = AuthCodeSpotify::from_token(token);
    spotify.config.token_refreshing = true;

    let result = match spotify
        .current_playback(
            None,
            Some(&vec![AdditionalType::Track, AdditionalType::Episode]),
        )
        .await
    {
        Ok(context) => context,
        Err(err) => return Response::error(format!("UNAUTHORIZED \n {:?}",err), StatusCode::UNAUTHORIZED.as_u16()),
    };

    let playing_context = match result {
        Some(result) => result,
        None => return Response::error("No Playing context", StatusCode::BAD_REQUEST.as_u16()),
    };

    let item = match playing_context.item {
        Some(item) => item,
        None => return Response::error("No Playing context", StatusCode::BAD_REQUEST.as_u16()),
    };

    let mut response = match item {
        PlayableItem::Track(track) => Response::from_json(&track).unwrap(),
        PlayableItem::Episode(episode) => Response::from_json(&episode).unwrap(),
    };

    response.headers_mut()
        //add CORS
        .append("Access-Control-Allow-Origin", "https://tweetdeck.twitter.com")?;
    response.headers_mut()
        .append("access-control-allow-credentials","true")?;
    response.headers_mut()
        .append("access-control-allow-methods","GET")?;
    Ok(response)
}
