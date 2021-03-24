use clap::{App, Arg, ArgGroup, ArgMatches};
use serde::{Deserialize, Serialize};

use crate::http;
use crate::settings::global_user::GlobalUser;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Migrations {
    pub migrations: Vec<MigrationConfig>,
}

impl Migrations {
    pub fn api_migration(
        &self,
        script_name: &str,
        account_id: &str,
        user: &GlobalUser,
    ) -> Result<ApiMigration, failure::Error> {
        let list_scripts_addr = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts",
            account_id,
        );

        let client = http::legacy_auth_client(user);
        let res = client.get(&list_scripts_addr).send()?;
        let scripts = res.json::<ListScriptsV4ApiResponse>()?.result;
        let script = scripts.iter().find(|script| script.name == script_name);

        match script {
            None
            | Some(ApiScript {
                name: _,
                migration_tag: None,
            }) => Self::coalesce_migrations(&self.migrations, None),
            Some(ApiScript {
                name: _,
                migration_tag: Some(tag),
            }) => {
                let mut iter = self
                    .migrations
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.tag.as_ref() == Some(&tag));

                match (iter.next(), iter.next()) {
                    (Some((i, _)), None) if i + 1 < self.migrations.len() => {
                        Self::coalesce_migrations(&self.migrations[i + 1..], Some(tag))
                    }
                    (Some(_), None) => Ok(ApiMigration {
                        old_tag: Some(tag.clone()),
                        new_tag: Some(tag.clone()),
                        migration: Migration::default(),
                    }),
                    (Some(_), Some(_)) => failure::bail!(
                        "Multiple migrations with tag {} were found in your wrangler.toml",
                        tag
                    ),
                    (None, _) => failure::bail!(
                        "The migration with tag {} was not found in your wrangler.toml.",
                        tag
                    ),
                }
            }
        }
    }

    fn coalesce_migrations(
        migrations: &[MigrationConfig],
        old_tag: Option<&str>,
    ) -> Result<ApiMigration, failure::Error> {
        if migrations.is_empty() {
            failure::bail!("cannot coalesce empty migrations");
        }
        log::info!(
            "coalescing migrations {:#?} after {:?}",
            migrations,
            old_tag
        );
        let mut new_classes = migrations[0].migration.durable_objects.new_classes.to_vec();
        let mut deleted_classes = migrations[0]
            .migration
            .durable_objects
            .deleted_classes
            .to_vec();
        let mut renamed_classes = migrations[0]
            .migration
            .durable_objects
            .renamed_classes
            .to_vec();
        let mut transferred_classes = migrations[0]
            .migration
            .durable_objects
            .transferred_classes
            .to_vec();

        for mc in &migrations[1..] {
            // Renames will modify existing new, rename, and transfer migrations if they are present
            for rename in mc.migration.durable_objects.renamed_classes.iter() {
                if let Some(i) = new_classes.iter().position(|c| c == &rename.from) {
                    new_classes[i] = rename.to.clone();
                } else if let Some(existing) = renamed_classes
                    .iter_mut()
                    .find(|existing| existing.to == rename.from)
                {
                    existing.to = rename.to.clone();
                } else if let Some(existing) = transferred_classes
                    .iter_mut()
                    .find(|existing| existing.to == rename.from)
                {
                    existing.to = rename.to.clone();
                } else {
                    // rename does not modify an existing migration, add it to the final migration
                    renamed_classes.push(rename.clone());
                }
            }

            // deletes will remove existing new, rename, and transfer migrations if they are present
            for delete in mc.migration.durable_objects.deleted_classes.iter() {
                if let Some(i) = new_classes.iter().position(|c| c == delete) {
                    new_classes.remove(i);
                } else if let Some(i) = renamed_classes
                    .iter_mut()
                    .position(|existing| &existing.to == delete)
                {
                    renamed_classes.remove(i);
                } else if let Some(i) = transferred_classes
                    .iter_mut()
                    .position(|existing| &existing.to == delete)
                {
                    transferred_classes.remove(i);
                } else {
                    // delete does not modify an existing migration, add it to the final migration
                    deleted_classes.push(delete.clone());
                }
            }

            // The other migrations cannot modify an existing migration, so we can directly add them to the final migration
            new_classes.extend(mc.migration.durable_objects.new_classes.iter().cloned());
            transferred_classes.extend(
                mc.migration
                    .durable_objects
                    .transferred_classes
                    .iter()
                    .cloned(),
            );
        }
        let migration = ApiMigration {
            old_tag: old_tag.map(|t| t.to_owned()),
            new_tag: migrations.last().map(|c| c.tag.clone()).flatten(),
            migration: Migration {
                durable_objects: DurableObjectsMigration {
                    new_classes,
                    deleted_classes,
                    renamed_classes,
                    transferred_classes,
                },
            },
        };
        log::info!("final migration {:#?}", migration);
        Ok(migration)
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
struct ListScriptsV4ApiResponse {
    pub result: Vec<ApiScript>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
struct ApiScript {
    #[serde(rename = "id")]
    pub name: String,
    pub migration_tag: Option<String>,
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
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub new_classes: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deleted_classes: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub renamed_classes: Vec<RenameClass>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
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

    #[test]
    fn migration_coalescing() {
        let migrations = [
            MigrationConfig {
                tag: Some(String::from("1")),
                migration: Migration {
                    durable_objects: DurableObjectsMigration {
                        new_classes: vec![String::from("new1A")],
                        deleted_classes: vec![],
                        renamed_classes: vec![],
                        transferred_classes: vec![TransferClass {
                            from: String::from("old"),
                            from_script: String::from("old"),
                            to: String::from("transfer1B"),
                        }],
                    },
                },
            },
            MigrationConfig {
                tag: Some(String::from("2")),
                migration: Migration {
                    durable_objects: DurableObjectsMigration {
                        new_classes: vec![String::from("new2C")],
                        deleted_classes: vec![String::from("new1A"), String::from("class2Z")],
                        renamed_classes: vec![
                            RenameClass {
                                from: String::from("transfer1B"),
                                to: String::from("rename2B"),
                            },
                            RenameClass {
                                from: String::from("renameZ"),
                                to: String::from("rename2Z"),
                            },
                        ],
                        transferred_classes: vec![],
                    },
                },
            },
            MigrationConfig {
                tag: Some(String::from("3")),
                migration: Migration {
                    durable_objects: DurableObjectsMigration {
                        new_classes: vec![String::from("new3D")],
                        deleted_classes: vec![],
                        renamed_classes: vec![
                            RenameClass {
                                from: String::from("rename2B"),
                                to: String::from("rename3B"),
                            },
                            RenameClass {
                                from: String::from("new2C"),
                                to: String::from("rename3C"),
                            },
                        ],
                        transferred_classes: vec![],
                    },
                },
            },
        ];

        let final_migration = ApiMigration {
            old_tag: None,
            new_tag: Some(String::from("3")),
            migration: Migration {
                durable_objects: DurableObjectsMigration {
                    new_classes: vec![String::from("rename3C"), String::from("new3D")],
                    deleted_classes: vec![String::from("class2Z")],
                    renamed_classes: vec![RenameClass {
                        from: String::from("renameZ"),
                        to: String::from("rename2Z"),
                    }],
                    transferred_classes: vec![TransferClass {
                        from: String::from("old"),
                        from_script: String::from("old"),
                        to: String::from("rename3B"),
                    }],
                },
            },
        };

        assert_eq!(
            final_migration,
            Migrations::coalesce_migrations(&migrations, None).unwrap()
        );
    }
}
