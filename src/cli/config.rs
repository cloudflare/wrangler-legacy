use crate::commands;
use crate::settings::global_user::GlobalUser;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{interactive, styles};

use anyhow::Result;

pub fn configure(api_key: bool, no_verify: bool) -> Result<()> {
    let user: GlobalUser = if !api_key {
        // API Tokens are the default
        StdOut::billboard(&format!(
            concat!(
                "To find your API Token, go to {}\n",
                "and create it using the \"Edit Cloudflare Workers\" template.\n",
                "\n",
                "Consider using {} which only requires your Cloudflare username and password.\n",
                "\n",
                "If you are trying to use your Global API Key instead of an API Token\n",
                "{}, run {}."
            ),
            styles::url("https://dash.cloudflare.com/profile/api-tokens"),
            styles::highlight("`wrangler login`"),
            styles::warning("(Not Recommended)"),
            styles::highlight("`wrangler config --api-key`")
        ));
        let api_token: String = interactive::get_user_input("Enter API Token: ");
        GlobalUser::TokenAuth { api_token }
    } else {
        StdOut::billboard(&format!(concat!(
            "We don't recommend using your Global API Key!\n",
            "Please consider using an API Token instead.\n",
            "\n",
            "{}"),
            styles::url(
                "https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys",
            )
        ));
        let email: String = interactive::get_user_input("Enter Email: ");
        let api_key: String = interactive::get_user_input("Enter Global API Key: ");

        GlobalUser::GlobalKeyAuth { email, api_key }
    };

    commands::global_config(&user, !no_verify)
}
