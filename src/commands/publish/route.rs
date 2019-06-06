use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use log::info;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Route {
    script: Option<String>,
    pub pattern: String,
}

#[derive(Deserialize)]
struct RoutesResponse {
    result: Vec<Route>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectRoutes {
    routes: Vec<Route>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RoutesDiff {
    added: Vec<Route>,
    modified: Vec<Route>,
    // remotes: Vec<Route>, // TODO: turn into a HashMap for easy lookup by pattern
    // removed: Option<Vec<Route>>,
}

impl ProjectRoutes {
    pub fn new(project: &Project) -> Result<ProjectRoutes, failure::Error> {
        Ok(ProjectRoutes {
            routes: get_local_routes(project)
                .expect("Could not get local routes from wranger.toml"),
        })
    }

    // patterns returns all patterns listed in this config
    pub fn patterns(&self) -> Vec<String> {
        return self
            .routes
            .iter()
            .map(|r| r.pattern.clone())
            .collect::<Vec<String>>();
    }

    // publish syncs our locally defined routes with Cloudflare
    // creating, updating or deleting routes as necessary
    // TODO: needs to be rethought along with diff
    pub fn publish(&self, user: &GlobalUser, project: &Project) -> Result<(), failure::Error> {
        let diff = self.diff(user, project)?;
        // Add new routes
        for route in diff.added {
            create(user, project, &route)?;
        }
        // Modify existing routes (patterns pointing to a different script)
        for route in diff.modified {
            println!(
                "⚠️ Wrangler can't yet sync modified routes: '{pattern}' -> '{script}'",
                pattern = route.pattern,
                script = route.script.unwrap_or_default()
            )
        }
        Ok(())
    }

    // diffs local and remote diffs
    // TODO: the original code was designed to only add paths which might get messy
    // the whole route-sync logic needs to be rethought IMHO
    pub fn diff(&self, user: &GlobalUser, project: &Project) -> Result<RoutesDiff, failure::Error> {
        let remote_routes = get_remote_routes(user, project)?;
        // Routes to be created (defined locally, but no-matching remote pattern)
        let new_routes = self
            .routes
            .iter()
            .filter(|r| !remote_routes.iter().any(|rr| r.pattern == rr.pattern))
            .cloned()
            .collect::<Vec<Route>>();
        // Routes to be modified (pattern existed, but is routed to a different script)
        let modified_routes = self
            .routes
            .iter()
            .filter(|r| !remote_routes.iter().any(|rr| r.pattern == rr.pattern))
            .cloned()
            .collect::<Vec<Route>>();
        // Diff
        // TODO: explore delete (see above comments on publish/diff)
        Ok(RoutesDiff {
            added: new_routes,
            modified: modified_routes,
            // remotes: remote_routes,
        })
    }
}

impl Route {
    pub fn matches(&self, route: &Route) -> bool {
        self.pattern == route.pattern && self.script == route.script
    }
}

fn get_local_routes(project: &Project) -> Result<Vec<Route>, failure::Error> {
    let mut routes: Vec<Route> = vec![];
    if project.route.is_some() {
        // Single route
        routes = vec![Route {
            script: Some(project.name.to_string()),
            pattern: project.route.clone().unwrap().to_string(),
        }];
    } else if project.routes.is_some() && (project.routes.clone().unwrap().len() > 0) {
        // Build routes from project's HashMap

        for (pattern, script) in project.routes.clone().unwrap().iter() {
            routes.push(Route {
                script: Some(script.to_string()),
                pattern: pattern.to_string(),
            })
        }
    } else {
        failure::bail!(
            "You must provide a route or routes in your wrangler.toml before publishing!"
        );
    }
    return Ok(routes);
}

fn get_remote_routes(user: &GlobalUser, project: &Project) -> Result<Vec<Route>, failure::Error> {
    let routes_addr = get_routes_addr(project)?;

    let client = http::auth_client(user);

    let mut res = client.get(&routes_addr).send()?;

    if !res.status().is_success() {
        let msg = format!(
            "⛔ There was an error fetching your project's routes.\n Status Code: {}\n Msg: {}",
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }

    let routes_response: RoutesResponse = serde_json::from_str(&res.text()?)?;

    Ok(routes_response.result)
}

// Create or update a route
fn create(user: &GlobalUser, project: &Project, route: &Route) -> Result<(), failure::Error> {
    let client = http::auth_client(user);
    let body = serde_json::to_string(&route)?;

    let routes_addr = get_routes_addr(project)?;

    info!("Creating your route {:#?}", &route.pattern,);
    let mut res = client
        .post(&routes_addr)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()?;

    if !res.status().is_success() {
        let msg = format!(
            "⛔ There was an error creating your route.\n Status Code: {}\n Msg: {}",
            res.status(),
            res.text()?
        );
        failure::bail!(msg)
    }
    Ok(())
}

fn get_routes_addr(project: &Project) -> Result<String, failure::Error> {
    if let Some(zone_id) = &project.zone_id {
        return Ok(format!(
            "https://api.cloudflare.com/client/v4/zones/{}/workers/routes",
            zone_id
        ));
    }
    failure::bail!("You must provide a zone_id in your wrangler.toml.")
}
