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

    // Delete file if existent
    let config_path = get_global_config_path();
    if config_path.exists() {
        print!("Removing {}...", config_path.to_str().unwrap());
        fs::remove_file(config_path).expect("Failed to remove config file");
        println!(" success!");
    } else if has_auth {
        // (in)correct environment variables are set
        println!("No config file has been found. If you wish to unauntheticate `wrangler`, please unset your environment variables (e.g. \"CF_API_TOKEN\", \"CF_API_KEY\", or \"CF_EMAIL\").");
    }

    Ok(())
}

// Revoke refresh token, which also invalidates the current access token
pub fn revoke_token(user: &GlobalUser) -> Result<()> {
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL"),
        None,
    )
    .set_revocation_uri(
        RevocationUrl::new(REVOKE_URL.to_string()).expect("Invalid revocation endpoint URL"),
    )
    .set_auth_type(AuthType::RequestBody);

    let token_to_revoke = StandardRevocableToken::RefreshToken(RefreshToken::new(
        user.get_refresh_token().to_string(),
    ));
    match client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request(http_client)
    {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow::Error::new(err)),
    }
}
