use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

use crate::install;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Feature {
    Sites,
}

pub(crate) fn headers(feature: Option<Feature>) -> HeaderMap {
    let mut headers = HeaderMap::default();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&get_user_agent(feature)).unwrap(),
    );
    headers
}

fn get_user_agent(feature: Option<Feature>) -> String {
    let version = if install::target::DEBUG {
        "dev"
    } else {
        env!("CARGO_PKG_VERSION")
    };

    let mut agent = format!("wrangler/{}", version);
    if let Some(feature) = feature {
        agent.push('/');
        let feat = format!("{:?}", feature).to_lowercase();
        agent.push_str(&feat);
    }
    agent
}
