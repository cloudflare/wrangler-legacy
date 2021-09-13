use crate::login::{AUTH_URL, CLIENT_ID};
use crate::settings::get_global_config_path;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message::{Message, StdOut};

use anyhow::Result;

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthType, AuthUrl, ClientId, RefreshToken, RevocationUrl, StandardRevocableToken};

use std::fs;

static REVOKE_URL: &str = "https://dash.cloudflare.com/oauth2/revoke";

pub fn run() -> Result<()> {
    let mut has_auth = true;
    if let Ok(user) = GlobalUser::new() {
        match user {
            GlobalUser::OAuthTokenAuth { .. } => {
                // Set up OAuth client
                match revoke_token(&user) {
                    Ok(_) => StdOut::info("Wrangler is configured with an OAuth token. The token has been successfully revoked."),
                    Err(e) => anyhow::bail!(e),
                }
            }
            GlobalUser::ApiTokenAuth { .. } => {
                // API token can only be modified in the dashboard
                StdOut::info("Wrangler is configured with an API token. Please go to your dashboard if you would like to delete the API token.");
            }
            GlobalUser::GlobalKeyAuth { .. } => {
                // Global API key cannot be modified
                StdOut::info("Wrangler is configured with a Global API key.");
            }
        }
    } else {
        StdOut::info("Improperly configured or missing authentication method. Please see the documentation regarding `wrangler login` or `wrangler config` for authentication methods.");
        has_auth = false;
    }

    // Delete configuration file if existent
    let config_path = get_global_config_path();
    if config_path.exists() {
        let config_path_str = match config_path.to_str() {
            Some(path_name) => path_name,
            None => {
                log::debug!("Failed to convert config_path to str.");
                "configuration file"
            }
        };
        print!("Removing {}..", config_path_str);
        fs::remove_file(config_path)?;
        println!(" success!");
    } else if has_auth {
        // (in)correct environment variables are set
        println!("No config file has been found. If you wish to unauntheticate `wrangler`, please unset your environment variables (e.g. \"CF_API_TOKEN\", \"CF_API_KEY\", or \"CF_EMAIL\").");
    }

    Ok(())
}

// Revoke refresh token, which also invalidates the current access token
pub fn revoke_token(user: &GlobalUser) -> Result<()> {
    if let GlobalUser::OAuthTokenAuth { .. } = user {
        let auth_url = AuthUrl::new(AUTH_URL.to_string())?;
        let revoke_url = RevocationUrl::new(REVOKE_URL.to_string())?;

        let client = BasicClient::new(ClientId::new(CLIENT_ID.to_string()), None, auth_url, None)
            .set_revocation_uri(revoke_url)
            .set_auth_type(AuthType::RequestBody);

        let token_to_revoke = StandardRevocableToken::RefreshToken(RefreshToken::new(
            user.get_refresh_token().to_string(),
        ));
        if let Err(err) = client.revoke_token(token_to_revoke)?.request(http_client) {
            anyhow::bail!(err)
        }
    }
    Ok(())
}

// Invalidatess previous OAuth token if present
pub fn invalidate_oauth_token(command: String) {
    if let Ok(user) = GlobalUser::new() {
        // Try to invalidate previous token
        let result = revoke_token(&user);
        if result.is_err() {
            // A failure to invalidate a previous token should not block the user from being able to login with a new OAuth token
            log::debug!("Failed to invalidate OAuth token before {}", command);
        }
    }
}
