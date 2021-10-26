use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use globset::{Candidate, Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use path_slash::PathExt; // Path::to_slash()
use serde::{Deserialize, Serialize};

use super::binding::Binding;
use super::filestem_from_path;
use super::plain_text::PlainText;
use super::text_blob::TextBlob;
use super::wasm_module::WasmModule;
use super::UsageModel;

use crate::settings::toml::{
    migrations::ApiMigration, DurableObjectsClass, KvNamespace, ModuleRule, R2Bucket,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ServiceWorkerAssets {
    pub(crate) script_path: PathBuf,
    pub compatibility_date: Option<String>,
    pub compatibility_flags: Vec<String>,
    pub wasm_modules: Vec<WasmModule>,
    pub kv_namespaces: Vec<KvNamespace>,
    pub r2_buckets: Vec<R2Bucket>,
    pub durable_object_classes: Vec<DurableObjectsClass>,
    pub text_blobs: Vec<TextBlob>,
    pub plain_texts: Vec<PlainText>,
    pub usage_model: Option<UsageModel>,
}

impl ServiceWorkerAssets {
    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        for wm in &self.wasm_modules {
            let binding = wm.binding();
            bindings.push(binding);
        }
        for kv in &self.kv_namespaces {
            let binding = kv.binding();
            bindings.push(binding);
        }
        for r2 in &self.r2_buckets {
            let binding = r2.binding();
            bindings.push(binding);
        }
        for do_ns in &self.durable_object_classes {
            let binding = do_ns.binding();
            bindings.push(binding);
        }
        for blob in &self.text_blobs {
            let binding = blob.binding();
            bindings.push(binding);
        }
        for plain_text in &self.plain_texts {
            let binding = plain_text.binding();
            bindings.push(binding);
        }

        bindings
    }

    pub fn script_name(&self) -> Result<String> {
        filestem_from_path(&self.script_path).ok_or_else(|| {
            anyhow!(
                "filename should not be empty: {}",
                self.script_path.display()
            )
        })
    }

    pub fn script_path(&self) -> PathBuf {
        self.script_path.clone()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Module {
    pub path: PathBuf,
    pub module_type: ModuleType,
}

// All this macro does is generate some associated methods that return a value for each enum variant.
// as well as a .iter() associated function that lets you iterate over each module type
// This would be significantly longer without the macro, and makes adding new module types very easy.
// The format is [slice of globs (can be empty)] => VariantName("mime-type")
macro_rules! module_type {
    (pub enum $name:ident {
        $($globs:tt => $variant:ident($content_type:expr)),+,
    }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Hash, Serialize, PartialEq, PartialOrd, Eq, Ord)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),+
                }
            }

            pub fn content_type(&self) -> &'static str {
                match self {
                    $($name::$variant => $content_type),+
                }
            }

            pub fn default_globs(&self) -> &'static[&'static str] {
                match self {
                    $($name::$variant => &$globs),+
                }
            }

            pub fn iter() -> std::slice::Iter<'static, $name> {
                [$($name::$variant),+].iter()
            }
        }
    };
}

module_type! {
    pub enum ModuleType {
        ["**/*.mjs"] => ESModule("application/javascript+module"),
        ["**/*.js", "**/*.cjs"] => CommonJS("application/javascript"),
        [] => CompiledWasm("application/wasm"),
        [] => Text("text/plain"),
        [] => Data("application/octet-stream"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleConfig {
    pub main: String, // String since this is a module name, not a path.
    pub dir: PathBuf,
    rules: Vec<ModuleRule>,
}

pub struct ModuleManifest {
    pub main: String,
    pub modules: HashMap<String, Module>,
}

impl ModuleConfig {
    pub fn new(main: &str, dir: &Path, rules: &Option<Vec<ModuleRule>>) -> ModuleConfig {
        ModuleConfig {
            main: main.to_string(),
            dir: dir.to_path_buf(),
            rules: rules.clone().unwrap_or_default(),
        }
    }

    pub fn get_modules(self) -> Result<ModuleManifest> {
        let matchers = build_type_matchers(self.rules)?;

        let candidates_vec = WalkBuilder::new(&self.dir)
            .standard_filters(false)
            .follow_links(true)
            .build()
            .collect::<Result<Vec<_>, _>>()?;
        let candidates = candidates_vec
            .iter()
            .filter(|e| e.path().is_file())
            .map(|e| e.path());

        Ok(ModuleManifest {
            main: self.main.to_owned(),
            modules: Self::make_module_manifest(candidates, &self.dir, &matchers)?,
        })
    }

    fn make_module_manifest<'a, P>(
        paths: impl Iterator<Item = &'a P>,
        upload_dir: &'a Path,
        matchers: &'a [ModuleMatcher],
    ) -> Result<HashMap<String, Module>>
    where
        P: AsRef<Path> + ?Sized + 'a,
    {
        let processed_paths = paths
            .map(|p| {
                let p = p.as_ref();
                p.strip_prefix(upload_dir).map(|p_stripped_prefix| {
                    let p_stripped_prefix: PathBuf = p_stripped_prefix.to_slash_lossy().into();
                    // we convert the path used for matching and names to a slash path
                    // so globs are the same on all platforms
                    // to_slash_lossy() strips non-unicode characters in the path on windows
                    // which couldn't be represented in JS anyways
                    (p, p_stripped_prefix)
                })
            })
            .collect::<Result<Vec<(&Path, PathBuf)>, _>>()?;

        let mut final_types: HashSet<ModuleType> = HashSet::new();
        let modules: HashMap<_, _> = processed_paths
            .iter()
            .filter_map(|(prefixed_path, path)| {
                final_types.clear();
                let candidate = Candidate::new(&path);

                let mut match_result = None;
                for ModuleMatcher {
                    globs,
                    matcher,
                    module_type,
                    fallthrough,
                } in matchers
                {
                    if final_types.contains(module_type) {
                        continue;
                    }
                    if !fallthrough {
                        // this rule had fallthrough disabled, so we shouldn't consider
                        // rules for this module type
                        final_types.insert(*module_type);
                    }
                    let matches = matcher.matches_candidate(&candidate);
                    match matches.len() {
                        0 => log::info!(
                            "{} skipped by rule {:?} => {}",
                            path.display(),
                            globs,
                            module_type.name(),
                        ),
                        _ => {
                            let matched_globs = globs
                                .iter()
                                .enumerate()
                                .filter_map(
                                    |(i, g)| if matches.contains(&i) { Some(g) } else { None },
                                )
                                .collect::<Vec<_>>();
                            log::info!(
                                "{} matched by these globs {:?} => {}",
                                path.display(),
                                matched_globs,
                                module_type.name(),
                            );
                            let module_name = format!("./{}", path.display());
                            match_result = Some((
                                module_name,
                                Module {
                                    path: prefixed_path.to_path_buf(),
                                    module_type: *module_type,
                                },
                            ));
                            break;
                        }
                    }
                }

                match_result
            })
            .collect();

        Ok(modules)
    }
}

struct ModuleMatcher {
    globs: Vec<String>,
    matcher: GlobSet,
    module_type: ModuleType,
    fallthrough: bool,
}

fn new_glob(glob: &str) -> Result<Glob, globset::Error> {
    // we want to configure some defaults for all glob matches we make
    GlobBuilder::new(glob)
        .literal_separator(true)
        // we convert windows \ paths to / before matching
        // so glob rules work on all platforms,
        // so we need to force-enable backslash_escape as on windows it's disabled by default
        .backslash_escape(true)
        .build()
}

fn build_type_matchers(rules: Vec<ModuleRule>) -> Result<Vec<ModuleMatcher>> {
    let mut matchers = rules
        .into_iter()
        .map(|r| {
            let mut builder = GlobSetBuilder::new();

            for glob in &r.globs {
                let glob = new_glob(glob)?;
                builder.add(glob);
            }

            Ok(ModuleMatcher {
                globs: r.globs,
                matcher: builder.build()?,
                module_type: r.module_type,
                fallthrough: r.fallthrough,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    ModuleType::iter().try_for_each::<_, Result<(), globset::Error>>(|t| {
        let mut builder = GlobSetBuilder::new();
        for glob in t.default_globs() {
            builder.add(new_glob(glob)?);
        }
        matchers.push(ModuleMatcher {
            globs: t.default_globs().iter().map(|&g| g.to_owned()).collect(),
            matcher: builder.build().expect("default glob to be valid"),
            module_type: *t,
            fallthrough: false,
        });
        Ok(())
    })?;

    Ok(matchers)
}

pub struct ModulesAssets {
    pub compatibility_date: Option<String>,
    pub compatibility_flags: Vec<String>,
    pub manifest: ModuleManifest,
    pub kv_namespaces: Vec<KvNamespace>,
    pub r2_buckets: Vec<R2Bucket>,
    pub durable_object_classes: Vec<DurableObjectsClass>,
    pub migration: Option<ApiMigration>,
    pub text_blobs: Vec<TextBlob>,
    pub plain_texts: Vec<PlainText>,
    pub usage_model: Option<UsageModel>,
}

impl ModulesAssets {
    #[allow(clippy::too_many_arguments)] // TODO: refactor?
    pub fn new(
        compatibility_date: Option<String>,
        compatibility_flags: Vec<String>,
        manifest: ModuleManifest,
        kv_namespaces: Vec<KvNamespace>,
        r2_buckets: Vec<R2Bucket>,
        durable_object_classes: Vec<DurableObjectsClass>,
        migration: Option<ApiMigration>,
        text_blobs: Vec<TextBlob>,
        plain_texts: Vec<PlainText>,
        usage_model: Option<UsageModel>,
    ) -> Result<Self> {
        Ok(Self {
            compatibility_date,
            compatibility_flags,
            manifest,
            kv_namespaces,
            r2_buckets,
            durable_object_classes,
            migration,
            text_blobs,
            plain_texts,
            usage_model,
        })
    }

    pub fn bindings(&self) -> Vec<Binding> {
        let mut bindings = Vec::new();

        // Bindings that refer to a `part` of the uploaded files
        // in the service-worker format, are now modules.

        for kv in &self.kv_namespaces {
            let binding = kv.binding();
            bindings.push(binding);
        }
        for r2 in &self.r2_buckets {
            let binding = r2.binding();
            bindings.push(binding);
        }
        for class in &self.durable_object_classes {
            let binding = class.binding();
            bindings.push(binding);
        }
        for plain_text in &self.plain_texts {
            let binding = plain_text.binding();
            bindings.push(binding);
        }

        bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn init() {
        // enable INFO logging in tests
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();
    }

    // The below macros implement a simple DSL for specifying successful test results
    // the first entry should be an expression that evaluates to a ModuleConfig
    // the rest follow the following two patterns:
    // (full path on disk) => None
    //   -> indicates the path wasn't included in the upload
    // (full path on disk) => (module name, module type)
    //   -> indicates the path was included in the upload with a given module name and type

    macro_rules! test_success {
        (
            $config:expr;
            $($path:literal => $result:tt),+
        ) => {
            let mut expected_output: HashMap<String, Module> = HashMap::new();
            let mut paths: Vec<&Path> = Vec::new();

            macro_rules! test_data {
                ($path2:literal => None) => {
                    paths.push(Path::new($path2));
                };
                ($path2:expr => ($name:expr, $variant:ident)) => {{
                    paths.push(Path::new($path2));
                    expected_output.insert(
                        $name.to_string(),
                        Module {
                            path: $path2.into(),
                            module_type: ModuleType::$variant,
                        },
                    );
                }};
            }

            $(test_data!($path => $result));+;

            let matchers = build_type_matchers($config.rules)?;
            let modules = ModuleConfig::make_module_manifest(paths.into_iter(), &$config.dir, &matchers)?;

            assert_eq!(modules, expected_output);
            Ok(())
        }
    }

    #[test]
    fn default_globs() -> Result<()> {
        init();
        test_success! {
            ModuleConfig {
                main: r"./foo/bar/index.mjs".to_string(),
                dir: r"/worker/dist".into(),
                rules: Vec::new(),
            };
            r"/worker/dist/foo/bar/index.mjs" => (r"./foo/bar/index.mjs", ESModule),
            r"/worker/dist/bar.js" => (r"./bar.js", CommonJS),
            r"/worker/dist/foo/baz.cjs" => (r"./foo/baz.cjs", CommonJS),
            r"/worker/dist/wat.txt" => None,
            r"/worker/dist/wat.bin" => None,
            r"/worker/dist/code.wasm" => None,
            r"/worker/dist/sourcemap.map" => None
        }
    }

    // The following two macros implement a simple DSL for specifying Vec<ModuleRule>
    // The first matches individual rules, and the second calls the first for each rule
    // The formats are:
    // [slice of globs (can be empty)] => (<ModuleType variant name>) -> rule with fallthrough = false
    // [slice of globs (can be empty)] => (<ModuleType variant name>, fallthrough) -> rule with fallthrough = true

    macro_rules! rule {
        ([$($glob:literal),*]) => { vec![$($glob.to_owned()),*] };
        ($globs:tt => ($variant:ident, fallthrough)) => {
            ModuleRule {
                globs: rule!($globs),
                module_type: ModuleType::$variant,
                fallthrough: true
            }
        };
        ($globs:tt => ($variant:ident)) => {
            ModuleRule {
                globs: rule!($globs),
                module_type: ModuleType::$variant,
                fallthrough: false
            }
        };
    }

    macro_rules! rules {
        [$($globs:tt => $rule:tt),+] => {
            vec![$(rule!($globs => $rule)),+]
        };
    }

    #[test]
    fn custom_globs() -> Result<()> {
        init();
        test_success! {
            ModuleConfig {
                main: r"./foo/bar/index.mjs".to_string(),
                dir: r"/worker/dist".into(),
                rules: rules![
                    ["js-is-module/**/*.js"] => (ESModule, fallthrough),
                    ["**/*.js"] => (CommonJS),
                    [] => (Data),
                    ["**/*.wasm"] => (CompiledWasm)
                ],
            };
            r"/worker/dist/foo/bar/index.mjs" => (r"./foo/bar/index.mjs", ESModule),
            r"/worker/dist/foo.js" => (r"./foo.js", CommonJS),
            r"/worker/dist/js-is-module/bar.js" => (r"./js-is-module/bar.js", ESModule),
            r"/worker/dist/js-is-module/inner/bat.js" => (r"./js-is-module/inner/bat.js", ESModule),
            r"/worker/dist/wont-match/js-is-module/inner/bat.js" => (r"./wont-match/js-is-module/inner/bat.js", CommonJS),
            r"/worker/dist/wat.txt" => None,
            r"/worker/dist/code.wasm" => (r"./code.wasm", CompiledWasm),
            r"/worker/dist/baz.cjs" => None,
            r"/worker/dist/wat.bin" => None,
            r"/worker/dist/sourcemap.map" => None
        }
    }

    #[test]
    fn invalid_globs_fail() {
        let rules = rules![
            ["[z-a].mjs"] => (ESModule)
        ];

        println!(
            "{:?}",
            build_type_matchers(rules)
                .err()
                .expect("error on invalid globs")
        );
    }
}
