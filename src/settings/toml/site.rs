use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::commands::generate::run_generate;
use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{KvNamespace, Target};

const SITE_ENTRY_POINT: &str = "workers-site";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub bucket: PathBuf,
    #[serde(rename = "entry-point")]
    entry_point: Option<PathBuf>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    #[serde(skip)]
    pub kv_namespace: Option<KvNamespace>,
}

impl Site {
    pub fn new(bucket: &str) -> Site {
        let mut site = Site::default();
        site.bucket = PathBuf::from(bucket);

        site
    }

    pub fn kv_namespace(
        &self,
        user: &GlobalUser,
        target: &mut Target,
    ) -> Result<KvNamespace, failure::Error> {
        if let Some(site_namespace) = &self.kv_namespace {
            Ok(site_namespace.to_owned())
        } else {
            let site_namespace_title = format!("__{}-{}", target.name, "workers_sites_assets");
            let site_namespace = kv::namespace::upsert(target, &user, &site_namespace_title)?;
            Ok(KvNamespace {
                id: site_namespace.id,
                binding: "__STATIC_CONTENT".to_string(),
                bucket: Some(self.bucket),
            })
        }
    }

    // if the user has configured `site.entry-point`, use that
    // as the build directory. Otherwise use the default const
    // SITE_ENTRY_POINT
    pub fn entry_point(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        Ok(current_dir.join(
            self.entry_point
                .to_owned()
                .unwrap_or_else(|| PathBuf::from(SITE_ENTRY_POINT)),
        ))
    }

    pub fn scaffold_worker(&self) -> Result<(), failure::Error> {
        let entry_point = &self.entry_point()?;
        let template = "https://github.com/cloudflare/worker-sites-init";

        if !entry_point.exists() {
            log::info!("Generating a new Workers Sites project");
            run_generate(entry_point.file_name().unwrap().to_str().unwrap(), template)?;

            // This step is to prevent having a git repo within a git repo after
            // generating the scaffold into an existing project.
            fs::remove_dir_all(&entry_point.join(".git"))?;
        }

        Ok(())
    }
}

impl Default for Site {
    fn default() -> Site {
        Site {
            bucket: PathBuf::new(),
            entry_point: Some(PathBuf::from(SITE_ENTRY_POINT)),
            include: None,
            exclude: None,
            kv_namespace: None,
        }
    }
}
