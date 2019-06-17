use crate::http;
use std::collections::HashMap;
use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use log::info;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Route {
    id: Option<String>, // ID if remote
    script: Option<String>,
    pub pattern: String,
}

#[derive(Deserialize)]
struct RoutesResponse {
    result: Vec<Route>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectRoutes {
    script: String,
    routes: Vec<Route>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RoutesDiff {
    added: Vec<Route>,
    modified: Vec<Route>,
    deleted: Vec<Route>,
}

impl ProjectRoutes {
    pub fn new(project: &Project) -> Result<ProjectRoutes, failure::Error> {
        Ok(ProjectRoutes {
            script: project.name.clone(),
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
    pub fn publish(&self, user: &GlobalUser, project: &Project) -> Result<(), failure::Error> {
        let diff = self.diff(user, project)?;

        // Then: delete, modify, add

        // Delete old routes (that pointed to this script)
        for route in diff.deleted {
            // TODO: delete
            println!(
                "⚠️ Wrangler can't yet delete routes: '{pattern}' [#{id}] -> '{script}'",
                pattern = route.pattern,
                script = route.script.unwrap_or_default(),
                id = route.id.unwrap_or_default(),
            )
        }

        // Modify existing routes (patterns pointing to a different script)
        // TODO:
        // 1. Safely modify when pattern pointed to this script or "nothing" (i.e: on/off)
        // 2. Require a --force flag when pattern pointed to another script
        for route in diff.modified {
            println!(
                "⚠️ Wrangler can't yet sync modified routes: '{pattern}' -> '{script}'",
                pattern = route.pattern,
                script = route.script.unwrap_or_default()
            )
        }

        // Add new routes
        for route in diff.added {
            create(user, project, &route)?;
        }

        Ok(())
    }

    // diffs local and remote routes
    pub fn diff(&self, user: &GlobalUser, project: &Project) -> Result<RoutesDiff, failure::Error> {
        let remotes = get_remote_routes(user, project)?;

        // Hashmaps for easier lookups
        let remotes_map = routes_by_pattern(&remotes);
        let locals_map = routes_by_pattern(&self.routes);

        // Shorthand to tell if a route points to this project
        let same = |r: &Route| match &r.script {
            Some(name) => *name == self.script,
            None => false,
        };

        // Routes to be created (defined locally, but no-matching remote pattern)
        let added = _where(&self.routes, |r| !remotes_map.contains_key(&r.pattern));
        // Routes to be modified (pattern existed, but is routed to a different script)
        let modified = _where(&self.routes, |&r| !same(r) && remotes_map.contains_key(&r.pattern));
        // Remote routes to be deleted (that aren't specified in this script's config)
        let deleted = _where(&remotes, |&r| same(r) && !locals_map.contains_key(&r.pattern));

        // Diff
        Ok(RoutesDiff {
            added: added,
            modified: modified,
            deleted: deleted,
        })
    }
}

impl Route {
    pub fn matches(&self, route: &Route) -> bool {
        self.pattern == route.pattern && self.script == route.script
    }
}

fn _where(routes: &Vec<Route>, f: impl Fn(&&Route)->bool) -> Vec<Route> {
    return routes
        .iter()
        .filter(f)
        .cloned()
        .collect::<Vec<Route>>();
}

fn routes_by_pattern(routes: &Vec<Route>) -> HashMap<String, Route> {
    return routes.iter().fold(HashMap::new(), |mut acc, route| {
        acc.insert(route.pattern.clone(), route.clone());
        return acc;
    });
}

fn get_local_routes(project: &Project) -> Result<Vec<Route>, failure::Error> {
    let mut routes: Vec<Route> = vec![];

    // No specified routes
    if !(project.route.is_some() || project.routes_on.is_some()) {
        failure::bail!(
            "You must provide a route or routes in your wrangler.toml before publishing!"
        );
    }

    // All routes will point to the project's script (AKA it's name)
    let script_name = project.name.to_string();

    // Single route
    if project.route.is_some() {
        return Ok(vec![Route {
            id: None,
            script: Some(script_name),
            pattern: project.route.clone().unwrap().to_string(),
        }]);;
    }

    // Enabled routes
    for pattern in project.routes_on.clone().unwrap_or_default().iter() {
        routes.push(Route {
            id: None,
            script: Some(script_name.clone()),
            pattern: pattern.to_string(),
        })
    }
    // Disabled routes
    for pattern in project.routes_off.clone().unwrap_or_default().iter() {
        routes.push(Route {
            id: None,
            script: None,
            pattern: pattern.to_string(),
        })
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
