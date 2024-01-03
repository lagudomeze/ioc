use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DependencyValue {
    String(String),
    Object {
        version: Option<String>,
        features: Option<Vec<String>>,
    },
}

impl DependencyValue {
    fn auto_ioc(&self) -> bool {
        if let DependencyValue::Object {
            features: Some(features),
            ..
        } = self
        {
            features.contains(&"auto-ioc".to_string())
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CargoToml {
    dependencies: HashMap<String, DependencyValue>,
}

impl CargoToml {
    pub(crate) fn current() -> CargoToml {
        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut cargo_path = PathBuf::new();
        cargo_path.push(cargo_manifest_dir);
        cargo_path.push("Cargo.toml");

        Self::from(cargo_path)
    }

    pub(crate) fn from<P: AsRef<Path>>(path: P) -> CargoToml {
        let cargo_toml_raw = fs::read_to_string(&path).unwrap();
        toml::from_str(&cargo_toml_raw).unwrap()
    }

    pub(crate) fn mod_names<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        self.dependencies
            .iter()
            .filter(|(_, v)| v.auto_ioc())
            .map(|(k, _)| k.as_str())
    }
}