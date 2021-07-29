use url::Url;
use uuid::Uuid;

use super::http_method::HttpMethod;

pub struct RequestPayload {
    pub method: HttpMethod,
    pub https: u8,
    pub session: String,
    pub domain: String,
    pub browser_url: String,
    pub service_url: String,
    pub body: Option<String>,
}

impl RequestPayload {
    pub fn create(method: HttpMethod, url: Url, body: Option<String>) -> RequestPayload {
        let session = Uuid::new_v4().to_simple().to_string();

        let https = if url.scheme() == "https" { 1 } else { 0 };
        let protocol = format!("{}://", url.scheme());

        let domain = url.domain().unwrap().to_string();
        let path = url.path().to_string();

        let query = match url.query() {
            Some(query) => format!("?{}", query),
            None => "".to_string(),
        };

        let browser_url = format!("{}{}{}{}", protocol, domain, path, query);
        let service_url = format!(
            "{}{}{}",
            "https://00000000000000000000000000000000.cloudflareworkers.com", path, query
        );

        RequestPayload {
            method,
            https,
            session,
            domain,
            browser_url,
            service_url,
            body,
        }
    }

    pub fn cookie(&self, script_id: &str) -> String {
        format!(
            "__ew_fiddle_preview={}{}{}{}",
            script_id, self.session, self.https, self.domain
        )
    }
}
