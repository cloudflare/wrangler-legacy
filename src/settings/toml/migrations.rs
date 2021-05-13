use clap::{App, Arg, ArgGroup, ArgMatches};
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

impl MigrationConfig {
    pub fn add_to_app<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app
        .arg(
            Arg::with_name("new_class")
            .help("allow durable objects to be created from a class in your script")
            .long("new-class")
            .takes_value(true)
            .number_of_values(1)
            .value_name("class name")
            .multiple(true)
        )
        .arg(
            Arg::with_name("delete_class")
            .help("delete all durable objects associated with a class in your script")
            .long("delete-class")
            .takes_value(true)
            .number_of_values(1)
            .value_name("class name")
            .multiple(true)
        )
        .arg(
            Arg::with_name("rename_class")
            .help("rename a durable object class")
            .long("rename-class")
            .takes_value(true)
            .number_of_values(2)
            .value_names(&["from class", "to class"])
            .multiple(true)
        )
        .arg(
            Arg::with_name("transfer_class")
            .help("transfer all durable objects associated with a class in another script to a class in this script")
            .long("transfer-class")
            .takes_value(true)
            .number_of_values(3)
            .value_names(&["from script", "from class", "to class"])
            .multiple(true)
        )
        .group(
            ArgGroup::with_name("migrations")
            .args(&["new_class", "delete_class", "rename_class", "transfer_class"])
            .multiple(true)
        )
    }

    pub fn from_matches(matches: &ArgMatches) -> Option<Self> {
        if matches.is_present("migrations") {
            let new_classes = matches
                .values_of("new_class")
                .iter_mut()
                .flatten()
                .map(|c| c.to_owned())
                .collect();
            let deleted_classes = matches
                .values_of("delete_class")
                .iter_mut()
                .flatten()
                .map(|c| c.to_owned())
                .collect();
            let renamed_classes = matches
                .values_of("rename_class")
                .iter_mut()
                .flatten()
                .scan(None, |last_value: &mut Option<&str>, current_value| {
                    let rename = if let Some(last) = last_value {
                        let last = last.to_owned();
                        *last_value = None;
                        Some(RenameClass {
                            from: last,
                            to: current_value.to_owned(),
                        })
                    } else {
                        *last_value = Some(current_value);
                        None
                    };
                    Some(rename)
                })
                .filter_map(|m| m)
                .collect();
            let transferred_classes = matches
                .values_of("transfer_class")
                .iter_mut()
                .flatten()
                .scan(Vec::<&str>::new(), |values, current_value| {
                    let mut transfer = None;
                    values.push(current_value);
                    if values.len() == 3 {
                        transfer = Some(TransferClass {
                            from_script: values[0].to_owned(),
                            from: values[1].to_owned(),
                            to: values[2].to_owned(),
                        });
                        values.clear();
                    }
                    Some(transfer)
                })
                .filter_map(|m| m)
                .collect();

            let migration = Migration {
                durable_objects: DurableObjectsMigration {
                    new_classes,
                    deleted_classes,
                    renamed_classes,
                    transferred_classes,
                },
            };

            log::info!("adhoc migration {:#?}", migration);

            Some(MigrationConfig {
                tag: None,
                migration,
            })
        } else {
            None
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rename_class(tag: &str) -> RenameClass {
        RenameClass {
            from: format!("renameFrom{}", tag),
            to: format!("renameTo{}", tag),
        }
    }

    fn transfer_class(tag: &str) -> TransferClass {
        TransferClass {
            from: format!("transferFromClass{}", tag),
            from_script: format!("transferFromScript{}", tag),
            to: format!("transferToClass{}", tag),
        }
    }

    #[test]
    fn adhoc_migration_parsing() {
        use crate::util::ApplyToApp;
        let mut app = App::new("test").apply(MigrationConfig::add_to_app);

        let matches = app
            .get_matches_from_safe_borrow(&[
                "test",
                "--new-class",
                "newA",
                "--new-class",
                "newB",
                "--delete-class",
                "deleteA",
                "--delete-class",
                "deleteB",
                "--rename-class",
                "renameFromA",
                "renameToA",
                "--rename-class",
                "renameFromB",
                "renameToB",
                "--transfer-class",
                "transferFromScriptA",
                "transferFromClassA",
                "transferToClassA",
                "--transfer-class",
                "transferFromScriptB",
                "transferFromClassB",
                "transferToClassB",
            ])
            .unwrap();

        let migration = MigrationConfig::from_matches(&matches);

        assert_eq!(
            migration,
            Some(MigrationConfig {
                tag: None,
                migration: Migration {
                    durable_objects: DurableObjectsMigration {
                        new_classes: vec![String::from("newA"), String::from("newB")],
                        deleted_classes: vec![String::from("deleteA"), String::from("deleteB")],
                        renamed_classes: vec![rename_class("A"), rename_class("B")],
                        transferred_classes: vec![transfer_class("A"), transfer_class("B")],
                    }
                }
            })
        );
    }
}
