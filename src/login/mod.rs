pub mod http;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;

use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RefreshToken, Scope, TokenResponse, TokenUrl,
};

use crate::terminal::{interactive, open_browser};

use crate::cli::login::SCOPES_LIST;
use crate::commands::config::global_config;
use crate::commands::logout::invalidate_oauth_token;
use crate::login::http::http_server_get_params;
use crate::settings::{get_global_config_path, global_user::GlobalUser};

pub static CLIENT_ID: &str = "54d11594-84e4-41aa-b438-e81b8fa78ee7";
pub static AUTH_URL: &str = "https://dash.cloudflare.com/oauth2/auth";
static TOKEN_URL: &str = "https://dash.cloudflare.com/oauth2/token";
static CALLBACK_URL: &str = "http://localhost:8976/oauth/callback";

pub fn run(scopes: Option<&[String]>) -> Result<()> {
    let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
    let token_url = TokenUrl::new(TOKEN_URL.to_string())?;
    let redirect_url = RedirectUrl::new(CALLBACK_URL.to_string())?;

    // Create oauth2 client
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url)
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
    let runtime = tokio::runtime::Runtime::new()?;
    let params_response = runtime.block_on(http_server_get_params())?;
    let params_values: Vec<&str> = params_response.split_whitespace().collect();
    if params_values.is_empty() {
        anyhow::bail!(display_error_info(
            "Failed to receive authorization code from local HTTP server."
        ))
    }

    // Check if user has given consent, or if an error has been encountered
    let response_status = params_values[0];
    if response_status == "denied" {
        anyhow::bail!("Consent denied. You must grant consent to Wrangler in order to login. If you don't want to do this consider using `wrangler config`")
    } else if response_status == "error" {
        anyhow::bail!(display_error_info(
            "Failed to receive authorization code from local HTTP server."
        ))
    }

    // Get authorization code and CSRF state
    if params_values.len() != 3 {
        anyhow::bail!(display_error_info(
            "Failed to receive authorization code and/or csrf state from local HTTP server."
        ))
    }

    let auth_code = params_values[1];
    let recv_csrf_state = params_values[2];

    // Check CSRF token to ensure redirect is legit
    let recv_csrf_state = CsrfToken::new(recv_csrf_state.to_string());
    if recv_csrf_state.secret() != csrf_state.secret() {
        anyhow::bail!(display_error_info("Redirect URI CSRF state check failed."))
    }

    // Exchange authorization token for access token
    let token_response = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request(http_client)?;

    // Get access token expiration time
    let expires_in = match TokenResponse::expires_in(&token_response) {
        Some(time) => time,
        None => anyhow::bail!(display_error_info(
            "Failed to receive access_token expire time."
        )),
    };

    let expiration_time_value = match Utc::now().checked_add_signed(Duration::from_std(expires_in)?)
    {
        Some(time) => time,
        None => anyhow::bail!(display_error_info(
            "Failed to calculate access_token expiration time."
        )),
    };
    let expiration_time_value = expiration_time_value.to_rfc3339();

    let refresh_token_value = match token_response.refresh_token() {
        Some(token) => token,
        None => anyhow::bail!(display_error_info("Failed to receive refresh token.")),
    };

    // Configure user with new token
    let user = GlobalUser::OAuthTokenAuth {
        oauth_token: TokenResponse::access_token(&token_response)
            .secret()
            .to_string(),
        refresh_token: refresh_token_value.secret().to_string(),
        expiration_time: expiration_time_value,
    };

    // Invalidate previous OAuth token if present
    invalidate_oauth_token("`wrangler login`".to_string());
    global_config(&user, false)?;

    Ok(())
}

// Refresh an expired access token
pub fn check_update_oauth_token(user: &mut GlobalUser) -> Result<()> {
    if let GlobalUser::OAuthTokenAuth { .. } = user {
        log::debug!("Refreshing access token..");

        let expiration_time = DateTime::parse_from_rfc3339(user.get_expiration_time())?;
        let current_time = Utc::now();
        // Note: duration can panic if the time elapsed (in seconds) cannot be stored in i64
        let duration = current_time.signed_duration_since(expiration_time);

        // Access token expired
        // Refresh token before 20 seconds from actual expiration time to avoid minute details
        if duration.num_seconds() >= -20 {
            let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
            let token_url = TokenUrl::new(TOKEN_URL.to_string())?;
            let redirect_url = RedirectUrl::new(CALLBACK_URL.to_string())?;

            // Create oauth2 client
            let client = BasicClient::new(
                ClientId::new(CLIENT_ID.to_string()),
                None,
                auth_url,
                Some(token_url),
            )
            .set_redirect_uri(redirect_url)
            .set_auth_type(AuthType::RequestBody);

            // Exchange refresh token with new access token
            let refresh_token = user.get_refresh_token();
            let token_response = client
                .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
                .request(http_client)?;

            // Set new access token
            let access_token = token_response.access_token().secret();
            user.set_oauth_token(access_token.to_string());

            // Set new refresh token
            let new_refresh_token = token_response.refresh_token();
            if let Some(token) = new_refresh_token {
                user.set_refresh_token(token.secret().to_string());
            } else {
                anyhow::bail!(display_error_info(
                    "Failed to receive refresh token while updating access token."
                ))
            }

            // Set new expiration time
            let expires_in = match token_response.expires_in() {
                Some(time) => time,
                None => anyhow::bail!(display_error_info(
                    "Failed to receive access_token expire time while updating access token."
                )),
            };
            let expiration_time =
                match Utc::now().checked_add_signed(Duration::from_std(expires_in)?) {
                    Some(time) => time,
                    None => anyhow::bail!(display_error_info(
                    "Failed to calculate access_token expiration time while updating access token."
                )),
                };
            let expiration_time = expiration_time.to_rfc3339();
            user.set_expiration_time(expiration_time);

            // Update configuration file on disk
            let config_file = get_global_config_path();
            user.to_file(&config_file)?
        }
    }
    Ok(())
}

// Adds additional info besides an error message
pub fn display_error_info(error_msg: &str) -> String {
    let error_info = format!("{} Please run `wrangler login` again. If the error persists, consider reporting the issue through `wrangler report`.", error_msg);
    error_info
}
