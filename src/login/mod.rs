use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;

use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};

use std::env; // TODO: remove

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

use tokio::sync::mpsc;

use anyhow::Result;

use crate::terminal::{interactive, open_browser};

use crate::commands::config::global_config;
use crate::settings::global_user::{GlobalUser, TokenType};

// HTTP Server request handler
async fn handle_callback(req: Request<Body>, tx: mpsc::Sender<String>) -> Result<Response<Body>> {
    match req.uri().path() {
        // Endpoint given when registering oauth client
        "/oauth/callback" => {
            // Get authorization code from request
            let params = req
                .uri()
                .query()
                .map(|v| url::form_urlencoded::parse(v.as_bytes()))
                .unwrap();

            // Get authorization code and csrf state
            let mut params_values: Vec<String> = Vec::with_capacity(2);
            for (key, value) in params {
                if key == "code" || key == "state" {
                    params_values.push(value.to_string());
                }
            }

            // Send authorization code back
            let params_values_str = format!("{} {}", params_values[0], params_values[1]);
            tx.send(params_values_str).await?;

            // TODO: Ask if there is anything more official
            let response = Response::new("You have authorized wrangler. Please close this window and return to your terminal! :)".into());

            Ok(response)
        }
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();

            Ok(response)
        }
    }
}

pub fn run() -> Result<()> {
    // -------------------------
    // Temporary authentication
    // TODO: Remove when ready 
    let env_key = "CLIENT_ID";
    let client_id = match env::var(env_key) {
        Ok(value) => value,
        Err(_) => panic!("client_id not provided")
    };

    // -------------------------

    // Create oauth2 client
    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new("https://dash.staging.cloudflare.com/oauth2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(
            TokenUrl::new("https://dash.staging.cloudflare.com/oauth2/token".to_string())
                .expect("Invalid token endpoint URL"),
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8976/oauth/callback".to_string())
            .expect("Invalid redirect URL"),
    )
    .set_auth_type(AuthType::RequestBody);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Create URL for user
    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("account:read".to_string()))
        .add_scope(Scope::new("user:read".to_string()))
        .add_scope(Scope::new("workers:write".to_string())) // TODO: double check that this is needed
        .add_scope(Scope::new("workers_kv:write".to_string()))
        .add_scope(Scope::new("workers_routes:write".to_string()))
        .add_scope(Scope::new("workers_scripts:write".to_string()))
        .add_scope(Scope::new("workers_tail:read".to_string()))
        .add_scope(Scope::new("zone:read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let browser_permission =
        interactive::confirm("Allow Wrangler to open a page in your browser?")?;
    if !browser_permission {
        anyhow::bail!("In order to log in you must allow Wrangler to open your browser. If you don't want to do this consider using `wrangler config`");
    }
    open_browser(auth_url.as_str())?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);

    // Create and start listening for redirect on local HTTP server
    let server_fn_gen = |tx: mpsc::Sender<String>| {
        service_fn(move |req: Request<Body>| {
            let tx_clone = tx.clone();
            handle_callback(req, tx_clone)
        })
    };

    let service = make_service_fn(move |_socket: &AddrStream| {
        let tx_clone = tx.clone();
        async move { Ok::<_, hyper::Error>(server_fn_gen(tx_clone)) }
    });

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.spawn(async {
        let addr = ([127, 0, 0, 1], 8976).into();

        let server = Server::bind(&addr).serve(service);
        server.await.unwrap();
    });

    // Receive authorization code and csrf state from HTTP server
    let params_values = runtime.block_on(async { rx.recv().await.unwrap() });
    let mut params_values_iter = params_values.split_whitespace();
    let auth_code = params_values_iter
        .next()
        .expect("Failed to retrieve authorization code");
    let recv_csrf_state = params_values_iter
        .next()
        .expect("Failed to retrieve csrf state");

    // Check CSRF token to ensure redirect is legit
    let recv_csrf_state = CsrfToken::new(recv_csrf_state.to_string());
    if recv_csrf_state.secret() != csrf_state.secret() {
        anyhow::bail!(
            "Redirect URI CSRF state check failed. Received: {}, expected: {}",
            recv_csrf_state.secret(),
            csrf_state.secret()
        );
    }

    // Exchange authorization token for access token
    let token_response = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request(http_client)
        .expect("Failed to retrieve access token");

    // Configure user with new token
    let user = GlobalUser::TokenAuth {
        token_type: TokenType::Oauth,
        value: TokenResponse::access_token(&token_response)
            .secret()
            .to_string(),
    };
    global_config(&user, false)?;

    Ok(())
}
