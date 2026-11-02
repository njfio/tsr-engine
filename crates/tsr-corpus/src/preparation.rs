//! Explicit controller-owned preparation for lockless source snapshots.
use crate::digest;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    #[serde(default)]
    pub additional_locks: std::collections::BTreeMap<String, LockSpec>,
    pub version: u32,
    pub source_trees: Vec<String>,
    pub repository_identity: String,
    pub image: String,
    pub lock_digest: String,
    pub generated_by: String,
    pub rust_test_environment: bool,
    #[serde(default)]
    pub build_workspace: bool,
    pub test_argv: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockSpec {
    pub lock_digest: String,
    pub generated_by: String,
}
pub fn full_test_argv() -> Vec<String> {
    [
        "cargo",
        "test",
        "--locked",
        "--offline",
        "--workspace",
        "--no-fail-fast",
        "-j",
        "1",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
#[derive(Clone)]
pub struct Preparation {
    spec: Spec,
    id: String,
    lock: Vec<u8>,
    raw: Vec<u8>,
    additional: std::collections::BTreeMap<String, Vec<u8>>,
}
impl Preparation {
    pub fn spec(&self) -> &Spec {
        &self.spec
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn lock(&self, tree: &str) -> Option<&[u8]> {
        if !self.spec.source_trees.iter().any(|s| s == tree) {
            return None;
        }
        Some(
            self.additional
                .get(tree)
                .map(Vec::as_slice)
                .unwrap_or(&self.lock),
        )
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !meta.file_type().is_file() || meta.len() > 1_048_576 {
            return Err("preparation must be a bounded regular file".into());
        }
        let raw = fs::read(path).map_err(|e| e.to_string())?;
        let spec: Spec = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        if spec.version != 1
            || spec.source_trees.is_empty()
            || spec
                .source_trees
                .iter()
                .any(|s| s.len() != 40 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
            || spec.repository_identity.is_empty()
            || spec.generated_by.is_empty()
            || spec.test_argv != full_test_argv()
            || spec.additional_locks.iter().any(|(tree, lock)| {
                !spec.source_trees.contains(tree) || lock.generated_by.is_empty()
            })
        {
            return Err("invalid versioned preparation specification".into());
        }
        let lock = crate::mine::read_cas(
            &path.parent().unwrap_or(Path::new(".")).join("cas"),
            &spec.lock_digest,
        )
        .map_err(|e| e.to_string())?;
        if lock.is_empty() || lock.len() > 8_388_608 {
            return Err("invalid prepared lock size".into());
        }
        let mut additional = std::collections::BTreeMap::new();
        for (tree, entry) in &spec.additional_locks {
            let bytes = crate::mine::read_cas(
                &path.parent().unwrap_or(Path::new(".")).join("cas"),
                &entry.lock_digest,
            )
            .map_err(|e| e.to_string())?;
            if bytes.is_empty() || bytes.len() > 8_388_608 {
                return Err("invalid additional lock size".into());
            }
            additional.insert(tree.clone(), bytes);
        }
        Ok(Self {
            additional,
            id: digest(&raw),
            spec,
            lock,
            raw,
        })
    }
    pub fn apply(&self, repo: &Path, snapshot: &str, dest: &Path) -> Result<Vec<u8>, String> {
        if snapshot.is_empty() || !snapshot.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("preparation requires exact source object".into());
        }
        let result = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["rev-parse", "--verify", &format!("{snapshot}^{{tree}}")])
            .output()
            .map_err(|e| e.to_string())?;
        let tree = String::from_utf8_lossy(&result.stdout).trim().to_string();
        if !result.status.success() || !self.spec.source_trees.contains(&tree) {
            return Err("source tree is not approved by preparation".into());
        }
        let path = dest.join("Cargo.lock");
        // Existing tracked files, symlinks and dangling symlinks are never replaced.
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("preparation refuses existing Cargo.lock".into()),
        }
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        let lock = self.lock(&tree).ok_or("missing tree lock")?;
        file.write_all(lock).map_err(|e| e.to_string())?;
        serde_json::to_vec(&serde_json::json!({"version":1,"preparation_id":self.id,"source_tree":tree,"lock_digest":digest(lock),"generated_by":self.spec.additional_locks.get(&tree).map(|l|l.generated_by.as_str()).unwrap_or(&self.spec.generated_by),"image":self.spec.image,"repository_identity":self.spec.repository_identity,"overlay":"Cargo.lock","original_lock":"absent"})).map_err(|e|e.to_string())
    }
    pub fn export(&self, out: &Path) -> Result<(), String> {
        fs::create_dir_all(out.join("cas")).map_err(|e| e.to_string())?;
        crate::mine::write_cas(&out.join("cas"), &self.id, &self.raw).map_err(|e| e.to_string())?;
        crate::mine::write_cas(&out.join("cas"), &self.spec.lock_digest, &self.lock)
            .map_err(|e| e.to_string())?;
        for bytes in self.additional.values() {
            crate::mine::write_cas(&out.join("cas"), &digest(bytes), bytes)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
/// Bind a task's identity to explicit dependency preparation; legacy tasks retain their IDs.
pub fn task_id(base: String, preparation: Option<&str>) -> String {
    preparation.map_or_else(
        || base.clone(),
        |p| digest(&serde_json::to_vec(&(base.as_str(), p)).expect("identity")),
    )
}

/// Operator input. This command records an already generated lock; it never resolves dependencies.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default)]
    pub additional_snapshots: Vec<AdditionalSnapshot>,
    pub repo: std::path::PathBuf,
    pub snapshot: String,
    pub lock: std::path::PathBuf,
    pub repository_identity: String,
    pub image: String,
    pub generated_by: String,
    pub rust_test_environment: bool,
    #[serde(default)]
    pub build_workspace: bool,
    pub test_argv: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalSnapshot {
    pub snapshot: String,
    pub lock: std::path::PathBuf,
    pub generated_by: String,
}
pub fn record(request: Request, out: &Path) -> Result<String, String> {
    if request.snapshot.len() != 40 || !request.snapshot.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("exact source commit/tree required".into());
    }
    let meta = fs::symlink_metadata(&request.lock).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() || meta.len() > 8_388_608 {
        return Err("lock must be a bounded regular file".into());
    }
    let lock = fs::read(&request.lock).map_err(|e| e.to_string())?;
    let tree = Command::new("git")
        .arg("-C")
        .arg(&request.repo)
        .args([
            "rev-parse",
            "--verify",
            &format!("{}^{{tree}}", request.snapshot),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !tree.status.success() {
        return Err("source snapshot unavailable".into());
    }
    let mut spec = Spec {
        additional_locks: Default::default(),
        version: 1,
        source_trees: vec![String::from_utf8_lossy(&tree.stdout).trim().into()],
        repository_identity: request.repository_identity,
        image: request.image,
        lock_digest: digest(&lock),
        generated_by: request.generated_by,
        rust_test_environment: request.rust_test_environment,
        build_workspace: request.build_workspace,
        test_argv: request.test_argv,
    };
    fs::create_dir_all(out.join("cas")).map_err(|e| e.to_string())?;
    crate::mine::write_cas(&out.join("cas"), &spec.lock_digest, &lock)
        .map_err(|e| e.to_string())?;
    for extra in request.additional_snapshots {
        if extra.snapshot.len() != 40
            || !extra.snapshot.bytes().all(|b| b.is_ascii_hexdigit())
            || extra.generated_by.is_empty()
        {
            return Err("invalid additional snapshot provenance".into());
        }
        let meta = fs::symlink_metadata(&extra.lock).map_err(|e| e.to_string())?;
        if !meta.file_type().is_file() || meta.len() > 8_388_608 {
            return Err("invalid additional lock".into());
        }
        let bytes = fs::read(&extra.lock).map_err(|e| e.to_string())?;
        let result = Command::new("git")
            .arg("-C")
            .arg(&request.repo)
            .args([
                "rev-parse",
                "--verify",
                &format!("{}^{{tree}}", extra.snapshot),
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let tree = String::from_utf8_lossy(&result.stdout).trim().to_string();
        if !result.status.success() || spec.source_trees.contains(&tree) {
            return Err("unavailable or duplicate preparation tree".into());
        }
        let lock_digest = digest(&bytes);
        crate::mine::write_cas(&out.join("cas"), &lock_digest, &bytes)
            .map_err(|e| e.to_string())?;
        spec.source_trees.push(tree.clone());
        spec.additional_locks.insert(
            tree,
            LockSpec {
                lock_digest,
                generated_by: extra.generated_by,
            },
        );
    }
    let raw = serde_json::to_vec_pretty(&spec).map_err(|e| e.to_string())?;
    let path = out.join("preparation.json");
    use std::io::Write;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .and_then(|mut f| f.write_all(&raw))
        .map_err(|e| e.to_string())?;
    let preparation = Preparation::load(&path)?;
    preparation.export(out)?;
    Ok(preparation.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, std::path::PathBuf, Preparation, String) {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir(&repo).unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "fixture"],
            vec!["config", "user.email", "fixture@example.invalid"],
        ] {
            assert!(Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .status()
                .unwrap()
                .success());
        }
        fs::write(repo.join("Cargo.toml"), "[workspace]\n").unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["add", "Cargo.toml"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["-c", "core.hooksPath=/dev/null", "commit", "-qm", "fixture"])
            .status()
            .unwrap()
            .success());
        let tree = String::from_utf8(
            Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(["rev-parse", "HEAD^{tree}"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();
        let lock = dir.path().join("generated.lock");
        fs::write(&lock, "version = 4\n").unwrap();
        let out = dir.path().join("prep");
        record(
            Request {
                additional_snapshots: vec![],
                repo: repo.clone(),
                snapshot: tree.clone(),
                lock,
                repository_identity: "fixture://repo".into(),
                image: format!("sha256:{}", "a".repeat(64)),
                generated_by: "fixture, no dependency resolution".into(),
                rust_test_environment: true,
                build_workspace: false,
                test_argv: full_test_argv(),
            },
            &out,
        )
        .unwrap();
        let prep = Preparation::load(&out.join("preparation.json")).unwrap();
        (dir, repo, prep, tree)
    }
    #[test]
    fn fresh_lock_is_verified_pinned_and_never_replaces_existing_bytes() {
        let (dir, repo, prep, tree) = fixture();
        let copy = dir.path().join("copy");
        crate::mine::materialize(&repo, &tree, &copy).unwrap();
        let receipt = prep.apply(&repo, &tree, &copy).unwrap();
        assert_eq!(
            fs::read(copy.join("Cargo.lock")).unwrap(),
            prep.lock(&tree).unwrap()
        );
        assert!(String::from_utf8(receipt).unwrap().contains(&prep.id));
        assert!(prep
            .apply(&repo, &tree, &copy)
            .unwrap_err()
            .contains("existing"));
        fs::write(copy.join("Cargo.lock"), "changed").unwrap();
        assert_eq!(prep.lock(&tree).unwrap(), b"version = 4\n");
        assert_ne!(task_id("task".into(), Some(&prep.id)), "task");
    }
    #[test]
    fn corrupt_cas_wrong_tree_and_link_are_refused() {
        let (dir, repo, prep, tree) = fixture();
        let copy = dir.path().join("copy");
        crate::mine::materialize(&repo, &tree, &copy).unwrap();
        assert!(prep.apply(&repo, &"0".repeat(40), &copy).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("absent"), copy.join("Cargo.lock")).unwrap();
            assert!(prep.apply(&repo, &tree, &copy).is_err());
        }
        fs::write(
            dir.path().join("prep/cas").join(&prep.spec.lock_digest),
            "corrupt",
        )
        .unwrap();
        assert!(Preparation::load(&dir.path().join("prep/preparation.json")).is_err());
    }
    #[test]
    fn prepared_worker_uses_immutable_image_and_preserves_argv_boundaries() {
        let (_dir, _repo, prep, _tree) = fixture();
        let runner = crate::sealed::SealedRunner::new(
            crate::sealed::Runtime::Docker,
            prep.spec.image.clone(),
            "65534:65534".into(),
        )
        .unwrap()
        .with_preparation(prep)
        .unwrap();
        let argv = vec![
            "cargo".into(),
            "test".into(),
            "--".into(),
            "literal; $(no-host-command)".into(),
        ];
        let plan = runner
            .plan(Path::new("/copy"), &argv, "tsr-m3-test")
            .unwrap();
        assert!(plan.iter().any(|s| s == "--locked"));
        assert!(plan.iter().any(|s| s == "literal; $(no-host-command)"));
        assert_eq!(plan.last().unwrap(), "--test-threads=1");
    }
    #[test]
    fn full_suite_cannot_be_narrowed_by_preparation() {
        let (dir, _repo, _prep, _tree) = fixture();
        let path = dir.path().join("prep/preparation.json");
        let original = fs::read(&path).unwrap();
        for extra in ["--lib", "--exclude", "some_test", "--test", "-p"] {
            let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
            value["test_argv"]
                .as_array_mut()
                .unwrap()
                .push(extra.into());
            fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(Preparation::load(&path).is_err(), "accepted {extra}");
        }
    }
    #[test]
    fn history_trees_receive_their_own_recorded_locks() {
        let (dir, repo, _prep, first) = fixture();
        fs::write(repo.join("Cargo.toml"), "[workspace]\nresolver = \"2\"\n").unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["-c", "core.hooksPath=/dev/null", "commit", "-qam", "second"])
            .status()
            .unwrap()
            .success());
        let second = String::from_utf8(
            Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(["rev-parse", "HEAD^{tree}"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();
        let other = dir.path().join("other.lock");
        fs::write(&other, "version = 3\n").unwrap();
        let out = dir.path().join("pair");
        record(
            Request {
                repo: repo.clone(),
                snapshot: first.clone(),
                lock: dir.path().join("generated.lock"),
                repository_identity: "fixture://repo".into(),
                image: format!("sha256:{}", "a".repeat(64)),
                generated_by: "first resolution".into(),
                rust_test_environment: true,
                build_workspace: false,
                test_argv: full_test_argv(),
                additional_snapshots: vec![AdditionalSnapshot {
                    snapshot: second.clone(),
                    lock: other,
                    generated_by: "second independent resolution".into(),
                }],
            },
            &out,
        )
        .unwrap();
        let p = Preparation::load(&out.join("preparation.json")).unwrap();
        for (tree, want) in [(first, "version = 4\n"), (second, "version = 3\n")] {
            let copy = dir.path().join(&tree);
            crate::mine::materialize(&repo, &tree, &copy).unwrap();
            p.apply(&repo, &tree, &copy).unwrap();
            assert_eq!(fs::read_to_string(copy.join("Cargo.lock")).unwrap(), want);
            assert_eq!(p.lock(&tree).unwrap(), want.as_bytes());
        }
    }
}
