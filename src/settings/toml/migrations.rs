use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Migrations {
    pub migrations: Vec<MigrationConfig>,
}

impl Migrations {
    pub fn api_migration(&self) -> Result<ApiMigration, anyhow::Error> {
        // TODO: make api call to get most recent tag, and coalesce tags afterwards.
        // For now, migrations will only ever have a single adhoc migration in it.
        match &self.migrations {
            migrations if migrations.len() == 1 => Ok(ApiMigration {
                old_tag: None,
                new_tag: None,
                migration: migrations[0].migration.clone(),
            }),
            migrations => Self::coalesce_migrations(migrations),
        }
    }

    fn coalesce_migrations(_migrations: &[MigrationConfig]) -> Result<ApiMigration, anyhow::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MigrationConfig {
    pub tag: Option<String>,
    #[serde(flatten)]
    pub migration: Migration,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ApiMigration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_tag: Option<String>,
    #[serde(flatten)]
    pub migration: Migration,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Migration {
    #[serde(flatten)]
    pub durable_objects: DurableObjectsMigration,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DurableObjectsMigration {
    pub new_classes: Vec<String>,
    pub deleted_classes: Vec<String>,
    pub renamed_classes: Vec<RenameClass>,
    pub transferred_classes: Vec<TransferClass>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RenameClass {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TransferClass {
    pub from: String,
    pub from_script: String,
    pub to: String,
}
