pub mod http;

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;

use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RefreshToken, Scope, TokenResponse, TokenUrl,
};

use std::env; // TODO: remove

use anyhow::Result;
use futures::executor::block_on;

use crate::terminal::{interactive, open_browser};

use crate::cli::login::SCOPES_LIST;
use crate::commands::config::global_config;
use crate::login::http::http_server_get_params;
use crate::settings::{get_global_config_path, global_user::GlobalUser};

static AUTH_URL: &str = "https://dash.staging.cloudflare.com/oauth2/auth";
static TOKEN_URL: &str = "https://dash.staging.cloudflare.com/oauth2/token";
static CALLBACK_URL: &str = "http://localhost:8976/oauth/callback";

pub fn run(scopes: Option<&Vec<String>>) -> Result<()> {
    // -------------------------
    // Temporary authentication
    // TODO: Remove when ready
    let env_key = "CLIENT_ID";
    let client_id = match env::var(env_key) {
        Ok(value) => value,
        Err(_) => panic!("client_id not provided"),
    };

    // -------------------------

    // Create oauth2 client
    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL"),
        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL")),
    )
    .set_redirect_uri(RedirectUrl::new(CALLBACK_URL.to_string()).expect("Invalid redirect URL"))
    .set_auth_type(AuthType::RequestBody);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Create URL for user with the necessary scopes
    let mut client_state = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);

    if scopes.is_none() {
        // User did not provide any scopes
        for scope in SCOPES_LIST {
            client_state = client_state.add_scope(Scope::new(scope.to_string()));
        }
    } else {
        // User did provide some scopes
        for scope in scopes.unwrap() {
            client_state = client_state.add_scope(Scope::new(scope.to_string()));
        }
    }
    client_state = client_state.add_scope(Scope::new("offline_access".to_string()));
    let (auth_url, csrf_state) = client_state.url();

    // Navigate to authorization endpoint
    let browser_permission =
        interactive::confirm("Allow Wrangler to open a page in your browser?")?;
    if !browser_permission {
        anyhow::bail!("In order to log in you must allow Wrangler to open your browser. If you don't want to do this consider using `wrangler config`");
    }
    open_browser(auth_url.as_str())?;

    // Get authorization code and CSRF state from local HTTP server
    let params_response = match block_on(http_server_get_params()) {
        Ok(params) => params,
        Err(_) => anyhow::bail!("Failed to receive authorization code from local HTTP server"),
    };
    let params_values: Vec<&str> = params_response.split_whitespace().collect();
    if params_values.is_empty() {
        anyhow::bail!("Failed to receive authorization code from local HTTP server")
    }

    // Check if user has given consent, or if an error has been encountered
    let response_status = params_values[0];
    if response_status == "denied" {
        anyhow::bail!("Consent denied. You must grant consent to Wrangler in order to login. If you don't want to do this consider using `wrangler config`")
    } else if response_status == "error" {
        anyhow::bail!("Failed to receive authorization code from local HTTP server")
    }

    // Get authorization code and CSRF state
    if params_values.len() != 3 {
        anyhow::bail!(
            "Failed to receive authorization code and/or csrf state from local HTTP server"
        )
    }

    let auth_code = params_values[1];
    let recv_csrf_state = params_values[2];

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
    let user = GlobalUser::OAuthTokenAuth {
        oauth_token: TokenResponse::access_token(&token_response)
            .secret()
            .to_string(),
        refresh_token: TokenResponse::refresh_token(&token_response)
            .expect("Failed to receive refresh token")
            .secret()
            .to_string(),
    };
    global_config(&user, false)?;

    Ok(())
}

// Refresh an expired access token
pub fn update_oauth_token(user: &mut GlobalUser) -> Result<()> {
    let env_key = "CLIENT_ID";
    let client_id = match env::var(env_key) {
        Ok(value) => value,
        Err(_) => panic!("client_id not provided"),
    };

    // -------------------------

    // Create oauth2 client
    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        None,
        AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL"),
        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL")),
    )
    .set_redirect_uri(RedirectUrl::new(CALLBACK_URL.to_string()).expect("Invalid redirect URL"))
    .set_auth_type(AuthType::RequestBody);

    // Exchange refresh token with new access token
    let refresh_token = user.get_refresh_token();
    let token_response = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request(http_client)
        .expect("Failed to refresh OAuth access token");

    // Set new access token
    let access_token = token_response.access_token().secret();
    user.set_oauth_token(access_token.to_string());

    // Set new refresh token
    let new_refresh_token = token_response.refresh_token();
    if let Some(token) = new_refresh_token {
        user.set_refresh_token(token.secret().to_string());
    }

    // Update configuration file on disk
    let config_file = get_global_config_path();
    user.to_file(&config_file)?;

    Ok(())
}
