use cargo_metadata::{CargoOpt, DependencyKind, MetadataCommand};
use std::{
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
};

fn tttt() {
    let mut manifest: PathBuf = env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();
    manifest.push("Cargo.toml");

    let pkg_name = env::var("CARGO_PKG_NAME")
        .expect("CARGO_PKG_NAME not set");

    let metadata = MetadataCommand::new()
        .manifest_path(&manifest)
        .features(CargoOpt::AllFeatures)
        .exec()
        .expect("cargo metadata failed");


    let mut map = HashMap::new();

    for package in &metadata.packages {
        map.insert(package.name.as_str(), package);
    }

    let self_package = *map.get(pkg_name.as_str())
        .expect(format!("package {} not found", pkg_name).as_str());

    let mut bootstraps = Vec::new();

    for dep in &self_package.dependencies {
        if dep.kind == DependencyKind::Build {
            if let Some(package) = map.get(dep.name.as_str()) {
                if let Some(value) = package.metadata.get("build") {
                    if let Some(name) = value.as_str() {
                        bootstraps.push(format!("{}::{}", package.name, name));
                    }
                }
            }
        }
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("metadata");
    fs::write(dest_path, format!("{:#?}", bootstraps)).expect("TODO: panic message");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
