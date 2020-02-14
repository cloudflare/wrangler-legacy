use serde::{Deserialize, Serialize};

use cloudflare::endpoints::workers::WorkersRoute;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Route {
    pub id: Option<String>,
    pub script: Option<String>,
    pub pattern: String,
}

impl From<&WorkersRoute> for Route {
    fn from(api_route: &WorkersRoute) -> Route {
        Route {
            id: Some(api_route.id.clone()),
            script: api_route.script.clone(),
            pattern: api_route.pattern.clone(),
        }
    }
}
