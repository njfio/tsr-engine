//! Protected Cargo workspace metadata. Cargo's locked offline resolver remains authoritative.
use super::*;
fn dependency_tables(spec: &ProjectSpec, value: &toml::Value, base: &str) -> Result<()> {
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    for (key, value) in table {
        if matches!(
            key.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        ) {
            let deps = value
                .as_table()
                .ok_or_else(|| anyhow::anyhow!("dependency table required"))?;
            ensure!(deps.len() <= 128, "dependency table bound");
            for dep in deps.values() {
                if let Some(fields) = dep.as_table() {
                    keys(
                        fields,
                        &[
                            "version",
                            "workspace",
                            "path",
                            "package",
                            "features",
                            "default-features",
                            "optional",
                        ],
                    )?;
                    if let Some(path) = fields.get("path") {
                        let path = path
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("dependency path required"))?;
                        ensure!(
                            !path.starts_with('/') && !path.contains('\\'),
                            "external dependency path refused"
                        );
                        let mut parts: Vec<_> = base.split('/').filter(|s| !s.is_empty()).collect();
                        for p in path.split('/') {
                            match p {
                                "." => (),
                                ".." => {
                                    ensure!(parts.pop().is_some(), "dependency escapes workspace");
                                }
                                other => parts.push(other),
                            }
                        }
                        let manifest = format!("{}/Cargo.toml", parts.join("/"));
                        ensure!(
                            valid_path(&manifest) && spec.protected.contains_key(&manifest),
                            "path dependency must name a protected workspace manifest"
                        );
                    }
                } else {
                    ensure!(dep.is_str(), "dependency must be a version or table");
                }
            }
        } else if matches!(key.as_str(), "target" | "workspace") || key.starts_with("cfg(") {
            dependency_tables(spec, value, base)?;
        }
    }
    Ok(())
}
pub(super) fn validate(spec: &ProjectSpec) -> Result<()> {
    let selected = spec.named_package.as_ref().unwrap();
    ensure!(
        spec.cargo
            && !selected.is_empty()
            && selected.len() <= 64
            && selected
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
        "invalid selected package"
    );
    ensure!(
        spec.cases.len() == 1,
        "named-library mode requires exactly one test"
    );
    ensure!(
        spec.editable
            .keys()
            .all(|p| spec.edit_windows.contains_key(p)),
        "workspace edits require protected windows"
    );
    let root: toml::Value = spec
        .protected
        .get("Cargo.toml")
        .ok_or_else(|| anyhow::anyhow!("root manifest required"))?
        .parse()?;
    let root_table = root
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("manifest table required"))?;
    keys(root_table, &["workspace"])?;
    let ws = root
        .get("workspace")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow::anyhow!("workspace required"))?;
    keys(
        ws,
        &[
            "members",
            "resolver",
            "exclude",
            "package",
            "dependencies",
            "lints",
        ],
    )?;
    let members = ws
        .get("members")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("members required"))?;
    ensure!(
        !members.is_empty() && members.len() <= 8,
        "workspace member bound"
    );
    dependency_tables(spec, &root, "")?;
    let mut found = false;
    let mut package_names = BTreeSet::new();
    for member in members {
        let member = member
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("literal member required"))?;
        let path = format!("{member}/Cargo.toml");
        ensure!(valid_path(&path), "unsafe member path");
        let manifest: toml::Value = spec
            .protected
            .get(&path)
            .ok_or_else(|| anyhow::anyhow!("protected member manifest missing: {path}"))?
            .parse()?;
        let table = manifest
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("manifest table required"))?;
        keys(
            table,
            &[
                "package",
                "lib",
                "bin",
                "dependencies",
                "dev-dependencies",
                "build-dependencies",
                "features",
                "lints",
                "target",
            ],
        )?;
        dependency_tables(spec, &manifest, member)?;
        // Restrict target-qualified dependencies too, regardless of the target spelling.
        if let Some(targets) = table.get("target").and_then(toml::Value::as_table) {
            for target in targets.values() {
                dependency_tables(spec, target, member)?;
            }
        }
        let package = manifest
            .get("package")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| anyhow::anyhow!("package required"))?;
        keys(
            package,
            &[
                "name",
                "version",
                "edition",
                "authors",
                "license",
                "repository",
                "rust-version",
                "description",
                "keywords",
                "categories",
                "build",
                "autolib",
                "autobins",
                "autotests",
                "autobenches",
                "autoexamples",
                "default-run",
                "readme",
            ],
        )?;
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("package name required"))?;
        package_names.insert(name.to_owned());
        if name == selected {
            ensure!(!found, "ambiguous package name");
            found = true;
            ensure!(
                package
                    .get("build")
                    .is_none_or(|v| v.as_bool() == Some(false))
                    && !spec.protected.contains_key(&format!("{member}/build.rs")),
                "selected package build script refused"
            );
            let lib = format!("{member}/src/lib.rs");
            ensure!(
                spec.editable.contains_key(&lib) || spec.protected.contains_key(&lib),
                "selected library missing"
            );
            if let Some(lib) = table.get("lib").and_then(toml::Value::as_table) {
                keys(lib, &["name", "path", "doctest"])?;
                ensure!(
                    lib.get("path")
                        .is_none_or(|v| v.as_str() == Some("src/lib.rs")),
                    "standard library path required"
                );
            }
        }
    }
    ensure!(found, "selected workspace package missing");
    let lock: toml::Value = spec.protected["Cargo.lock"].parse()?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("lock inventory missing"))?;
    ensure!(packages.len() <= 512, "lock inventory bound");
    for p in packages {
        if let Some(source) = p.get("source") {
            ensure!(
                source.as_str() == Some(REGISTRY),
                "only crates.io registry locks supported"
            );
            ensure!(
                p.get("checksum")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
                "registry checksum required"
            );
        } else {
            ensure!(
                p.get("name")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|n| package_names.contains(n)),
                "unadmitted local lock package"
            );
        }
    }
    Ok(())
}
