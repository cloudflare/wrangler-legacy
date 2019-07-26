mod create_namespace;

pub use create_namespace::create_namespace;

use cloudflare::auth::Credentials;
use cloudflare::response::{APIError, APISuccess};
use cloudflare::workerskv;
use cloudflare::HTTPAPIClient;

use crate::settings::global_user::GlobalUser;

impl From<GlobalUser> for Credentials {
    fn from(user: GlobalUser) -> Credentials {
        Credentials::User {
            key: user.api_key,
            email: user.email,
        }
    }
}

fn new_api_client(user: GlobalUser) -> HTTPAPIClient {
    let credentials = Credentials::from(user);

    HTTPAPIClient::new(credentials)
}

#[derive(Debug)]
pub struct KvNamespace {
    id: String,
    title: String,
}

impl From<APISuccess<workerskv::WorkersKVNamespace>> for KvNamespace {
    fn from(success: APISuccess<workerskv::WorkersKVNamespace>) -> KvNamespace {
        KvNamespace {
            id: success.result.id,
            title: success.result.title,
        }
    }
}

// TODO: Map these to Wrangler-relevant messages? or an Enum?
fn api_errors_to_message(errors: Vec<APIError>) -> String {
    let mut messages = Vec::new();
    for error in errors {
        match error.code {
            // https://api.cloudflare.com/#workers-kv-namespace-errors
            10001 => messages.push("service temporarily unavailable"),
            10002 => messages.push("missing CF-Ray header"),
            10003 => messages.push("missing account public ID"),
            10004 => messages.push("missing account tag"),
            10005 => messages.push("URL parameter account tag does not match JWT account tag"),
            10006 => messages.push("malformed account tag"),
            10007 => messages.push("malformed page argument"),
            10008 => messages.push("malformed per_page argument"),
            10009 => messages.push("key not found"),
            10010 => messages.push("malformed namespace"),
            10011 => messages.push("malformed namespace ID"),
            10012 => messages.push("malformed value"),
            10013 => messages.push("namespace not found"),
            10014 => messages.push("namespace already exists"),
            10015 => messages.push("missing account internal ID"),
            10016 => messages.push("malformed account internal ID"),
            10017 => messages.push("not entitled to use this endpoint"),
            10018 => messages.push("too many namespaces in this account"),
            10019 => messages.push("missing title"),
            10020 => messages.push("too many values in namespace"),
            10021 => messages.push("this namespace does not support the list-keys endpoint"),
            10022 => messages.push("too many requests"),
            10023 => messages.push("illegal key name"),
            10024 => messages.push("payload too large"),
            10025 => messages.push("path does not exist"),
            10026 => messages.push("not permitted"),
            10027 => messages.push("invalid storage gateway worker version"),
            10028 => messages.push("invalid limit argument"),
            10029 => messages.push("invalid request"),
            10030 => messages.push("key too long"),
            10031 => messages.push("invalid authorization"),
            10032 => messages.push("invalid tunnel type"),
            10033 => messages.push("invalid expiration"),
            10034 => messages.push("invalid expiration ttl"),
            10035 => messages.push("this namespace does not support the bulk endpoint"),
            10036 => messages.push("the request has improperly formatted permissions"),
            10037 => messages.push("the user lacks the permissions to perform this operation"),
            10038 => {
                messages.push("this namespace does not support the list-keys prefix parameter")
            }
            _ => messages.push("unknown error"),
        }
    }

    messages.join("\n")
}
