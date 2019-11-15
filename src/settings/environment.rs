use std::collections::HashMap;
use std::env;

use config::{ConfigError, Source, Value};

pub trait QueryEnvironment {
    fn get_var(&self, var: &'static str) -> Result<String, std::env::VarError>;

    fn empty(&self) -> Result<bool, failure::Error>;
}

#[derive(Clone, Debug)]
pub struct Environment {
    whitelist: Vec<&'static str>,
}

impl Environment {
    pub fn with_whitelist(whitelist: Vec<&'static str>) -> Self {
        Environment { whitelist }
    }
}

impl QueryEnvironment for Environment {
    fn get_var(&self, var: &'static str) -> Result<String, std::env::VarError> {
        env::var(var)
    }

    fn empty(&self) -> Result<bool, failure::Error> {
        let env = self.collect()?;

        Ok(env.is_empty())
    }
}

// Source trait implementation for use with Config::merge
// until config crate removal is complete. This is effectively a
// copy of the config crate's impl of Source for its Environment
// struct, but rather than pulling the whole environment in #collect,
// we pull in only whitelisted values, and rather than taking a custom
// prefix, we assume a prefix of `CF_`.
impl Source for Environment {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        let mut m = HashMap::new();
        let uri: String = "env".into();

        for key in &self.whitelist {
            if let Some(value) = env::var(key).ok() {
                // remove the `CF` prefix before adding to collection
                let prefix_pattern = "CF_";
                let key = &key[prefix_pattern.len()..];

                m.insert(key.to_lowercase(), Value::new(Some(&uri), value));
            }
        }

        Ok(m)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockEnvironment {
    vars: Vec<(&'static str, &'static str)>,
}

impl MockEnvironment {
    pub fn set(&mut self, key: &'static str, value: &'static str) -> &Self {
        self.vars.push((key, value));

        self
    }
}

impl QueryEnvironment for MockEnvironment {
    #[allow(unused_variables)]
    fn get_var(&self, var: &'static str) -> Result<String, std::env::VarError> {
        Ok("Some Mocked Result".to_string()) // Returns a mocked response
    }

    fn empty(&self) -> Result<bool, failure::Error> {
        Ok(self.vars.is_empty())
    }
}

// config::Source trait implementation for use with config::Config.merge
// until config crate removal is complete.
impl Source for MockEnvironment {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<HashMap<String, Value>, ConfigError> {
        let mut m = HashMap::new();
        let uri: String = "env".into();

        for (key, value) in &self.vars {
            // remove the `CF` prefix before adding to collection
            let prefix_pattern = "CF_";
            let key = &key[prefix_pattern.len()..];

            m.insert(key.to_lowercase(), Value::new(Some(&uri), *value));
        }

        Ok(m)
    }
}
