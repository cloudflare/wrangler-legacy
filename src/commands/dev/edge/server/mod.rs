mod http;
mod https;

pub use self::http::http;
pub use self::https::https;

use crate::commands::dev::utils::get_path_as_str;
use crate::commands::dev::Protocol;

use hyper::header::{HeaderName, HeaderValue};
use hyper::upgrade::OnUpgrade;
use hyper::{Body, Request};
use tokio::io::copy_bidirectional;

fn preview_request(
    mut parts: ::http::request::Parts,
    body: Body,
    preview_token: String,
    host: String,
    protocol: Protocol,
) -> Request<Body> {
    let path = get_path_as_str(&parts.uri);

    parts.headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_str(&host).expect("Could not create host header"),
    );

    parts.headers.insert(
        HeaderName::from_static("cf-workers-preview-token"),
        HeaderValue::from_str(&preview_token).expect("Could not create token header"),
    );

    parts.uri = match protocol {
        Protocol::Http => format!("http://{}{}", host, path),
        Protocol::Https => format!("https://{}{}", host, path),
    }
    .parse()
    .expect("Could not construct preview url");

    Request::from_parts(parts, body)
}

fn maybe_proxy_websocket(
    is_websocket: bool,
    client_on_upgrade: Option<OnUpgrade>,
    resp: &mut ::http::Response<Body>,
) {
    if is_websocket && resp.status() == 101 {
        if let (Some(client_on_upgrade), Some(upstream_on_upgrade)) = (
            client_on_upgrade,
            resp.extensions_mut().remove::<OnUpgrade>(),
        ) {
            tokio::spawn(async move {
                match tokio::try_join!(client_on_upgrade, upstream_on_upgrade) {
                    Ok((mut client_upgraded, mut server_upgraded)) => {
                        let proxy_future =
                            copy_bidirectional(&mut client_upgraded, &mut server_upgraded);
                        if let Err(err) = proxy_future.await {
                            log::warn!("could not proxy WebSocket: {}", err);
                        }
                    }
                    Err(e) => log::warn!("could not proxy WebSocket: {}", e),
                }
            });
        }
    }
}
