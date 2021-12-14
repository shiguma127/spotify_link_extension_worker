use http::StatusCode;
use rspotify::{
    clients::OAuthClient,
    model::{AdditionalType, PlayableItem},
    AuthCodeSpotify, Token,
};
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let cookie_string = match req.headers().get("cookie").unwrap() {
        Some(cookie) => cookie,
        None => return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16()),
    };
    let cookie = utils::get_cookie_from_string(cookie_string);
    let session_id = match cookie.get("session_id"){
        Some(session_id) => session_id,
        None => return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16()),
    };

    let kv = ctx.kv("SESSION_KV").unwrap();
    let token_json = match kv.get(session_id).await.unwrap() {
        Some(token_json) => token_json.as_string(),
        None => return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16()),
    };

    let token = match serde_json::from_str::<Token>(&token_json) {
        Ok(token) => token,
        Err(_) => {
            return Response::error(
                "INTERNAL_SERVER_ERROR",
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
        Err(_err) => return Response::error("UNAUTHORIZED", StatusCode::UNAUTHORIZED.as_u16()),
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
    Ok(response)
}
