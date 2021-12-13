use http::StatusCode;
use wasm_bindgen::UnwrapThrowExt;
use worker::{Request, Response, Result, RouteContext, Url};

use crate::utils;

pub fn handler(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let spotify = match utils::get_spotify_client(&ctx) {
        Ok(spotify) => spotify,
        Err(err) => {
            return Response::error(
                format!("Err:INTERNAL_SERVER_ERROR \n {:?}", err),
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            )
        }
    };
    let url = Url::parse(&spotify.get_authorize_url(false).unwrap_throw())?;
    Response::redirect_with_status(url, 301)
}
