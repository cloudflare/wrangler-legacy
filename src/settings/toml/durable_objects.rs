use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjects {
    pub classes: Option<Vec<DurableObjectsClass>>,
    #[serde(skip_deserializing)] // TODO remove when tagged migrations are fully supported
    pub migrations: Option<Vec<DurableObjectsMigrationConfig>>,
}

impl DurableObjects {
    pub fn merge_config_and_adhoc(
        config: DurableObjects,
        adhoc: DurableObjectsMigration,
    ) -> Result<DurableObjects, failure::Error> {
        // TODO: Right now we don't support migrations in config, so we just use what's in adhoc
        Ok(DurableObjects {
            classes: config.classes,
            migrations: Some(vec![DurableObjectsMigrationConfig {
                tag: None,
                migration: adhoc,
            }]),
        })
    }

    pub fn api_migrations(&self) -> Option<ApiDurableObjectsMigration> {
        // TODO: make api call to get most recent tag, and coalesce tags afterwards.
        // For now, migrations will only ever have a single adhoc migration in it.
        match &self.migrations {
            Some(migrations) if migrations.len() == 1 => Some(ApiDurableObjectsMigration {
                old_tag: None,
                new_tag: None,
                migration: migrations[0].migration.clone(),
            }),
            Some(_) => unimplemented!(),
            None => None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectsClass {
    pub binding: String,
    pub class_name: String,
    pub script_name: Option<String>,
}

impl DurableObjectsClass {
    pub fn binding(&self) -> Binding {
        Binding::new_durable_object_namespace(
            self.binding.clone(),
            self.class_name.clone(),
            self.script_name.clone(),
        )
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectsMigrationConfig {
    pub tag: Option<String>,
    #[serde(flatten)]
    pub migration: DurableObjectsMigration,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ApiDurableObjectsMigration {
    pub old_tag: Option<String>,
    pub new_tag: Option<String>,
    #[serde(flatten)]
    pub migration: DurableObjectsMigration,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectsMigration {
    pub new_classes: Vec<String>,
    pub deleted_classes: Vec<String>,
    pub unused_classes: Vec<String>,
}
