use binary_install::Cache;
use std::env;
use std::path::Path;

pub fn get_wrangler_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("WRANGLER_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("wrangler")
    }
}
