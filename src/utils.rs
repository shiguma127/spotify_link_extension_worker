use std::collections::{HashMap, HashSet};

use cfg_if::cfg_if;
use rspotify::{AuthCodeSpotify, Credentials, OAuth};
use worker::RouteContext;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn get_spotify_client(
    ctx: &RouteContext<()>,
) -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let credentials = Credentials {
        id: ctx.secret("spotify_client_id")?.to_string(),
        secret: Some(ctx.secret("spotify_client_secret")?.to_string()),
    };
    let oauth = OAuth {
        redirect_uri: String::from("http://127.0.0.1:8787/callback"),//ctx.var("REDIRECT_URI")?.to_string(),
        scopes: ctx
            .var("SCOPES")?
            .to_string()
            .split(',')
            .map(|s| s.to_string()) 
            .collect::<HashSet<_>>(),
        ..Default::default()
    };
    Ok(AuthCodeSpotify::new(credentials, oauth))
}

pub fn get_cookie_from_string(cookie_string: String) -> HashMap<String, String> {
    let mut cookie_map = HashMap::new();
    for cookie in cookie_string.split(';') {
        let mut cookie_parts = cookie.split('=');
        let key = cookie_parts.next().unwrap().trim().to_string();
        let value = cookie_parts.next().unwrap().trim().to_string();
        cookie_map.insert(key, value);
    }
    cookie_map
}
