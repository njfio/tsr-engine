//! Independent admission of bounded, single-function mutant candidates.
//! Evidence is controller-owned and lives outside the only directory mounted into the worker.
use crate::{run, CorpusTask, TaskKind};
use quote::ToTokens;
use std::{collections::BTreeSet, fs, io::Write, path::Path};
use tsr_corpus::{digest, mine, Limits, Runner};

pub fn qualify(manifest: &Path, out: &Path, runner: &dyn Runner) -> Result<usize, String> {
    let tasks = crate::read_tasks(manifest).map_err(|e| e.to_string())?;
    // Never overwrite an earlier cohort/evidence set.
    fs::create_dir(out).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("cas")).map_err(|e| e.to_string())?;
    let mut admitted = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join("tasks.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut reports = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join("admission.jsonl"))
        .map_err(|e| e.to_string())?;
    let cas = manifest.parent().unwrap_or(Path::new(".")).join("cas");
    let mut groups = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut count = 0;
    for task in tasks {
        if !ids.insert(task.id.clone()) {
            return Err("duplicate candidate id".into());
        }
        let evidence = out.join(&task.id);
        fs::create_dir(&evidence).map_err(|e| e.to_string())?;
        let result = qualify_one(&task, &cas, &evidence, runner, &groups);
        let report = match result {
            Ok(group) => {
                groups.insert(group.clone());
                for id in [task.mutation_diff.as_deref(), task.reversal_diff.as_deref()]
                    .into_iter()
                    .flatten()
                {
                    let bytes = mine::read_cas(&cas, id).map_err(|e| e.to_string())?;
                    mine::write_cas(&out.join("cas"), id, &bytes).map_err(|e| e.to_string())?;
                }
                if evidence.join("cas").is_dir() {
                    for entry in fs::read_dir(evidence.join("cas")).map_err(|e| e.to_string())? {
                        let name = entry
                            .map_err(|e| e.to_string())?
                            .file_name()
                            .to_string_lossy()
                            .to_string();
                        let bytes = mine::read_cas(&evidence.join("cas"), &name)
                            .map_err(|e| e.to_string())?;
                        mine::write_cas(&out.join("cas"), &name, &bytes)
                            .map_err(|e| e.to_string())?;
                    }
                }
                if let Some(id) = runner.preparation_id() {
                    let bytes = mine::read_cas(&out.join("cas"), id).map_err(|e| e.to_string())?;
                    let profile = out.join("preparation.json");
                    if profile.exists() {
                        if fs::read(&profile).map_err(|e| e.to_string())? != bytes {
                            return Err("conflicting cohort preparation".into());
                        }
                    } else {
                        fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(profile)
                            .and_then(|mut f| f.write_all(&bytes))
                            .map_err(|e| e.to_string())?;
                    }
                }
                writeln!(admitted, "{}", serde_json::to_string(&task).unwrap())
                    .map_err(|e| e.to_string())?;
                count += 1;
                serde_json::json!({"task_id":task.id,"accepted":true,"function_group":group,"protocol":"baseline-full-green/named-red/full-red/exact-reversal/full-green-v1","preparation_id":runner.preparation_id()})
            }
            Err(reason) => serde_json::json!({"task_id":task.id,"accepted":false,"reason":reason}),
        };
        writeln!(reports, "{report}").map_err(|e| e.to_string())?;
    }
    Ok(count)
}
fn qualify_one(
    task: &CorpusTask,
    cas: &Path,
    evidence: &Path,
    runner: &dyn Runner,
    groups: &BTreeSet<String>,
) -> Result<String, String> {
    if task.kind() != TaskKind::Mutant
        || !run::is_safe_repo(&task.repo)
        || task.snapshot().len() != 40
        || !task.snapshot().bytes().all(|b| b.is_ascii_hexdigit())
        || task.preparation_id.as_deref() != runner.preparation_id()
    {
        return Err("requires exact mutant snapshot and matching preparation".into());
    }
    let repo = Path::new(&task.repo);
    let copy = evidence.join("copy");
    let limits = Limits::default();
    let mut oracle =
        run::load_oracle_snapshot(repo, task.snapshot(), task).map_err(|e| e.to_string())?;
    if let Some(lock) = runner.prepared_lock(task.snapshot()) {
        if oracle.insert("Cargo.lock".into(), lock.to_vec()).is_some() {
            return Err("lock overlay replaced tracked file".into());
        }
    }
    let result = runner.provision(evidence, repo, task.snapshot(), &copy, &limits);
    if !result.succeeded() {
        return Err("provision failed".into());
    }
    runner.prepare(repo, task.snapshot(), &copy, evidence)?;
    let files = run::snapshot_files(repo, task.snapshot()).map_err(|e| e.to_string())?;
    let mut originals = std::collections::BTreeMap::new();
    for rel in files {
        if run::snapshot_is_symlink(repo, task.snapshot(), &rel).map_err(|e| e.to_string())? {
            return Err("linked source unsupported".into());
        }
        originals.insert(
            rel.clone(),
            run::snapshot_file(repo, task.snapshot(), &rel)
                .map_err(|e| e.to_string())?
                .ok_or("missing source")?,
        );
    }
    let mutation = mine::read_cas(
        cas,
        task.mutation_diff.as_deref().ok_or("missing mutation")?,
    )
    .map_err(|e| e.to_string())?;
    let reversal = mine::read_cas(
        cas,
        task.reversal_diff.as_deref().ok_or("missing reversal")?,
    )
    .map_err(|e| e.to_string())?;
    let mutation_path = patch_path(&copy, &mutation)?;
    if !originals.contains_key(&mutation_path)
        || !mutation_path.ends_with(".rs")
        || patch_path(&copy, &reversal)? != mutation_path
    {
        return Err("patches must touch one existing Rust source file".into());
    }
    let check = || -> Result<(), String> {
        if let Some(reason) = run::tamper_reason(&oracle, &copy, task).map_err(|e| e.to_string())? {
            return Err(reason);
        }
        Ok(())
    };
    let execute =
        |phase: &str, argv: Vec<String>, want: i32, test: Option<&str>| -> Result<(), String> {
            check()?;
            let result = runner.run(&copy, &argv, &limits);
            let id = tsr_corpus::persist_run(
                evidence,
                task.snapshot(),
                &argv,
                &result,
                runner.preparation_id(),
            )?;
            fs::write(
                evidence.join(format!("{phase}.json")),
                serde_json::to_vec(&serde_json::json!({"run":id,"exit":result.exit})).unwrap(),
            )
            .map_err(|e| e.to_string())?;
            check()?;
            if result.exit != want {
                return Err(format!(
                    "{phase}: expected completed exit {want}, got {}",
                    result.exit
                ));
            }
            if test.is_none() {
                let failed =
                    completed_suite(&result.stdout).map_err(|e| format!("{phase}: {e}"))?;
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stderr.contains("could not compile")
                    || stderr.contains("could not execute process")
                    || stderr.contains("process didn't exit successfully")
                {
                    return Err(format!("{phase}: build/infrastructure failure"));
                }
                if want == 101 && failed == 0 {
                    return Err(format!("{phase}: completed failure absent"));
                }
                if want == 0 && failed > 0 {
                    return Err(format!("{phase}: contradictory passing result"));
                }
            }
            if let Some(name) = test {
                let expected = format!(
                    "test {name} ... {}",
                    if want == 0 { "ok" } else { "FAILED" }
                );
                if !String::from_utf8_lossy(&result.stdout)
                    .lines()
                    .any(|l| l.trim() == expected)
                {
                    return Err(format!("{phase}: exact named result absent"));
                }
            }
            Ok(())
        };
    let verify_original = || -> Result<(), String> {
        for (rel, original) in &originals {
            if fs::read(copy.join(rel)).map_err(|e| e.to_string())? != *original {
                return Err(format!("source differs from trusted snapshot: {rel}"));
            }
        }
        Ok(())
    };
    verify_original()?;
    execute("baseline", suite_argv(runner, None), 0, None)?;
    verify_original()?;
    let paths_before = source_inventory(&copy)?;
    run::apply_diff(&copy, &mutation).map_err(|e| e.to_string())?;
    check()?;
    if source_inventory(&copy)? != paths_before {
        return Err("mutation added or deleted files".into());
    }
    let mut changed = Vec::new();
    for (rel, original) in &originals {
        let current = fs::read(copy.join(rel)).map_err(|e| e.to_string())?;
        if &current != original {
            changed.push((rel, original, current));
        }
    }
    if changed.len() != 1 {
        return Err("requires exactly one changed source file".into());
    }
    let (rel, original, current) = &changed[0];
    if !rel.ends_with(".rs") || rel.contains("/tests/") || rel.starts_with("tests/") {
        return Err("mutation is not production Rust".into());
    }
    let function = single_function(original, current)?;
    let group = digest(
        &serde_json::to_vec(&(
            runner.repository_identity().unwrap_or(&task.lineage),
            task.snapshot(),
            rel,
            function,
        ))
        .unwrap(),
    );
    if groups.contains(&group) {
        return Err("duplicate function group".into());
    }
    for (i, test) in task.failing_tests().iter().enumerate() {
        execute(
            &format!("red-{i}"),
            suite_argv(runner, Some(test)),
            101,
            Some(test),
        )?;
    }
    execute("mutated-full", suite_argv(runner, None), 101, None)?;
    run::apply_diff(&copy, &reversal).map_err(|e| e.to_string())?;
    if source_inventory(&copy)? != paths_before {
        return Err("reversal changed file inventory".into());
    }
    check()?;
    for (rel, original) in &originals {
        if fs::read(copy.join(rel)).map_err(|e| e.to_string())? != *original {
            return Err(format!("reversal did not restore {rel}"));
        }
    }
    for (i, test) in task.failing_tests().iter().enumerate() {
        execute(
            &format!("green-{i}"),
            suite_argv(runner, Some(test)),
            0,
            Some(test),
        )?;
    }
    execute("reversal-full", suite_argv(runner, None), 0, None)?;
    for (rel, original) in &originals {
        if fs::read(copy.join(rel)).map_err(|e| e.to_string())? != *original {
            return Err(format!("suite changed restored source {rel}"));
        }
    }
    if source_inventory(&copy)? != paths_before {
        return Err("suite changed source file inventory".into());
    }
    fs::write(
        evidence.join("integrity.json"),
        serde_json::to_vec(
            &serde_json::json!({"tracked_files_restored":originals.len(),"function_group":group}),
        )
        .unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(group)
}
fn suite_argv(runner: &dyn Runner, name: Option<&str>) -> Vec<String> {
    // Prepared workers expand cargo test to this same canonical suite themselves.
    let mut argv = if runner.preparation_id().is_some() {
        vec!["cargo".into(), "test".into()]
    } else {
        tsr_corpus::preparation::full_test_argv()
    };
    if let Some(name) = name {
        argv.extend(["--".into(), name.into(), "--exact".into()]);
    }
    argv
}
/// Validate that every started libtest group has a complete, consistent summary.
pub fn completed_suite(bytes: &[u8]) -> Result<usize, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let mut running = None;
    let mut groups = 0;
    let mut failed = 0usize;
    for line in text.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("running ") {
            let parts: Vec<_> = rest.split_whitespace().collect();
            if parts.len() == 2 && matches!(parts[1], "test" | "tests") {
                if running.is_some() {
                    return Err("unfinished test group".into());
                }
                running = Some(
                    parts[0]
                        .parse::<usize>()
                        .map_err(|_| "invalid test count")?,
                );
            }
        }
        if let Some(rest) = line.strip_prefix("test result: ") {
            let expected = running.take().ok_or("summary without test group")?;
            let (_, counts) = rest.split_once(". ").ok_or("incomplete summary")?;
            let fields: Vec<_> = counts.split(';').collect();
            if fields.len() < 4 {
                return Err("incomplete summary counts".into());
            }
            let mut total = 0usize;
            for (i, label) in ["passed", "failed", "ignored", "measured"]
                .iter()
                .enumerate()
            {
                let parts: Vec<_> = fields[i].split_whitespace().collect();
                if parts.len() != 2 || parts[1] != *label {
                    return Err("invalid summary count".into());
                }
                let count = parts[0].parse::<usize>().map_err(|_| "invalid count")?;
                total = total.checked_add(count).ok_or("count overflow")?;
                if i == 1 {
                    failed = failed.checked_add(count).ok_or("count overflow")?;
                }
            }
            if total != expected {
                return Err("summary does not complete group".into());
            }
            groups += 1;
        }
    }
    if groups == 0 || running.is_some() {
        return Err("missing completed full-suite evidence".into());
    }
    Ok(failed)
}
fn patch_path(copy: &Path, diff: &[u8]) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let canonical = copy.canonicalize().map_err(|e| e.to_string())?;
    let mut child = Command::new("git")
        .current_dir(&canonical)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env(
            "GIT_CEILING_DIRECTORIES",
            canonical.parent().unwrap_or(&canonical),
        )
        .args(["apply", "--numstat", "-z", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("missing patch stdin")?
        .write_all(diff)
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err("unparseable patch scope".into());
    }
    let text = String::from_utf8(result.stdout).map_err(|e| e.to_string())?;
    let rows: Vec<_> = text.split_terminator('\0').collect();
    if rows.len() != 1 {
        return Err("patch must affect exactly one path".into());
    }
    let fields: Vec<_> = rows[0].splitn(3, '\t').collect();
    if fields.len() != 3
        || fields[0].parse::<u64>().is_err()
        || fields[1].parse::<u64>().is_err()
        || !run::is_safe_oracle_path(fields[2])
    {
        return Err("unsupported patch path".into());
    }
    Ok(fields[2].into())
}
fn source_inventory(copy: &Path) -> Result<Vec<String>, String> {
    Ok(run::walk_files(copy)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|p| !p.starts_with("target/"))
        .collect())
}
fn benign(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}
fn tokens(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}
/// Restore the sole changed production function body in an AST, then require every other token
/// to be identical. Reject test modules, signatures, imports, attributes and multi-function edits.
fn single_function(before: &[u8], after: &[u8]) -> Result<String, String> {
    let a = syn::parse_file(std::str::from_utf8(before).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut b = syn::parse_file(std::str::from_utf8(after).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut changed = Vec::new();
    restore(&a.items, &mut b.items, "", false, &mut changed);
    if changed.len() != 1 || tokens(&a) != tokens(&b) {
        return Err(
            "requires one production function body and unchanged surrounding/test tokens".into(),
        );
    }
    Ok(changed.remove(0))
}
fn restore(
    a: &[syn::Item],
    b: &mut [syn::Item],
    prefix: &str,
    blocked: bool,
    changed: &mut Vec<String>,
) {
    for (a, b) in a.iter().zip(b) {
        match (a, b) {
            (syn::Item::Fn(a), syn::Item::Fn(b))
                if !blocked && benign(&a.attrs) && tokens(&a.block) != tokens(&b.block) =>
            {
                changed.push(format!("{prefix}::{}", a.sig.ident));
                b.block = a.block.clone();
            }
            (syn::Item::Impl(a), syn::Item::Impl(b)) if !blocked && benign(&a.attrs) => {
                for (x, y) in a.items.iter().zip(&mut b.items) {
                    if let (syn::ImplItem::Fn(x), syn::ImplItem::Fn(y)) = (x, y) {
                        if benign(&x.attrs) && tokens(&x.block) != tokens(&y.block) {
                            changed.push(format!(
                                "{prefix}::{}::{}",
                                tokens(&a.self_ty),
                                x.sig.ident
                            ));
                            y.block = x.block.clone();
                        }
                    }
                }
            }
            (syn::Item::Mod(a), syn::Item::Mod(b)) => {
                if let (Some((_, a_items)), Some((_, b_items))) = (&a.content, &mut b.content) {
                    restore(
                        a_items,
                        b_items,
                        &format!("{prefix}::{}", a.ident),
                        blocked || !benign(&a.attrs),
                        changed,
                    );
                }
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_one_production_body_can_change() {
        let a = b"fn f()->bool {true} fn g()->bool {true} #[cfg(test)] mod tests {fn t(){}}";
        assert!(single_function(
            a,
            b"fn f()->bool {false} fn g()->bool {true} #[cfg(test)] mod tests {fn t(){}}"
        )
        .is_ok());
        for b in [
            b"fn f()->bool {false} fn g()->bool {false} #[cfg(test)] mod tests {fn t(){}}"
                .as_slice(),
            b"fn f()->bool {true} fn g()->bool {true} #[cfg(test)] mod tests {fn t(){panic!()}}",
            b"fn f()->bool {false} fn g()->bool {true} #[cfg(test)] mod tests {fn t(){panic!()}}",
        ] {
            assert!(single_function(a, b).is_err());
        }
    }
    #[test]
    fn admission_exports_evidence_and_rejects_lifecycle_tampering() {
        use std::process::Command;
        struct Inert {
            preparation: Option<tsr_corpus::preparation::Preparation>,
            sabotage: u8,
            calls: std::cell::Cell<usize>,
        }
        impl Runner for Inert {
            fn preparation_id(&self) -> Option<&str> {
                self.preparation.as_ref().map(|p| p.id())
            }
            fn prepared_lock(&self, tree: &str) -> Option<&[u8]> {
                self.preparation.as_ref().and_then(|p| p.lock(tree))
            }
            fn prepare(
                &self,
                repo: &Path,
                tree: &str,
                copy: &Path,
                evidence: &Path,
            ) -> Result<(), String> {
                if let Some(p) = &self.preparation {
                    p.apply(repo, tree, copy)?;
                    p.export(evidence)?;
                }
                Ok(())
            }
            fn provision(
                &self,
                _: &Path,
                repo: &Path,
                snapshot: &str,
                dest: &Path,
                _: &Limits,
            ) -> tsr_corpus::RunResult {
                mine::materialize(repo, snapshot, dest).unwrap();
                tsr_corpus::RunResult::from_output(0, b"", b"", 0)
            }
            fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> tsr_corpus::RunResult {
                assert!(self.preparation.is_some() || argv.iter().any(|a| a == "--workspace"));
                let n = self.calls.get();
                self.calls.set(n + 1);
                if (self.sabotage == 1 && n == 0) || (self.sabotage == 2 && n == 4) {
                    fs::write(copy.join("src/lib.rs"), "changed by inert runner").unwrap();
                }
                if self.sabotage == 3 && n == 4 {
                    fs::write(copy.join("added.rs"), "extra").unwrap();
                }
                if self.sabotage == 4 && n == 2 {
                    return tsr_corpus::RunResult::from_output(
                        124,
                        b"bounded timeout",
                        b"",
                        600000,
                    );
                }
                if self.sabotage == 5 && n == 2 {
                    return tsr_corpus::RunResult::from_output(
                        101,
                        b"",
                        b"error: could not compile fixture",
                        1,
                    );
                }
                let red = fs::read_to_string(copy.join("src/lib.rs"))
                    .unwrap()
                    .contains("{false}");
                let named = argv.iter().any(|s| s == "tests::holds");
                tsr_corpus::RunResult::from_output(
                    if red { 101 } else { 0 },
                    if named {
                        if red {
                            b"test tests::holds ... FAILED\n"
                        } else {
                            b"test tests::holds ... ok\n"
                        }
                    } else if red {
                        b"running 1 test\ntest tests::holds ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
                    } else {
                        b"running 1 test\ntest tests::holds ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
                    },
                    b"",
                    0,
                )
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        let source="/// Production function.\npub fn f()->bool {true}\n#[cfg(test)] mod tests { #[test] fn holds(){assert!(super::f());} }\n";
        fs::write(repo.join("src/lib.rs"), source).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args([
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-c",
                    "user.name=fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                ])
                .args(args)
                .output()
                .unwrap();
            assert!(output.status.success());
            output.stdout
        };
        git(&["init", "-q"]);
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-qm", "fixture"]);
        let tree = String::from_utf8(git(&["rev-parse", "HEAD^{tree}"]))
            .unwrap()
            .trim()
            .to_string();
        fs::write(repo.join("src/lib.rs"), source.replace("{true}", "{false}")).unwrap();
        let mutation = git(&["diff", "--", "src/lib.rs"]);
        let reversal = git(&["diff", "-R", "--", "src/lib.rs"]);
        let cas = dir.path().join("cas");
        fs::create_dir(&cas).unwrap();
        for bytes in [&mutation, &reversal] {
            mine::write_cas(&cas, &digest(bytes), bytes).unwrap();
        }
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m3/batch/tasks.jsonl");
        let mut task = crate::read_tasks(&fixture).unwrap().remove(0);
        task.preparation_id = None;
        task.repo = repo.to_string_lossy().into();
        task.parent = None;
        task.commit = None;
        task.parent_snapshot = None;
        task.flipped_tests.clear();
        task.hidden_diff = None;
        task.mutated_snapshot = Some("descriptor".into());
        task.base_snapshot = Some(tree);
        task.mutation_diff = Some(digest(&mutation));
        task.reversal_diff = Some(digest(&reversal));
        task.failing_tests = vec!["tests::holds".into()];
        task.oracle_test_files = vec!["src/lib.rs".into()];
        let manifest = dir.path().join("tasks.jsonl");
        crate::write_tasks(&manifest, &[task.clone()]).unwrap();
        for sabotage in 0..6 {
            let out = dir.path().join(format!("out-{sabotage}"));
            let count = qualify(
                &manifest,
                &out,
                &Inert {
                    preparation: None,
                    sabotage,
                    calls: std::cell::Cell::new(0),
                },
            )
            .unwrap();
            assert_eq!(
                count,
                usize::from(sabotage == 0),
                "{}",
                fs::read_to_string(out.join("admission.jsonl")).unwrap()
            );
            if sabotage == 0 {
                assert_eq!(
                    crate::read_tasks(&out.join("tasks.jsonl")).unwrap().len(),
                    1
                );
                assert_eq!(
                    mine::read_cas(&out.join("cas"), task.mutation_diff.as_deref().unwrap())
                        .unwrap(),
                    mutation
                );
                assert!(out.join(&task.id).join("integrity.json").is_file());
            }
        }
        let lock = dir.path().join("prepared.lock");
        fs::write(&lock, "version = 4\n").unwrap();
        let profile_dir = dir.path().join("profile");
        let id = tsr_corpus::preparation::record(
            tsr_corpus::preparation::Request {
                additional_snapshots: vec![],
                repo: repo.clone(),
                snapshot: task.snapshot().into(),
                lock,
                repository_identity: "fixture://admission".into(),
                image: format!("sha256:{}", "a".repeat(64)),
                generated_by: "inert fixture".into(),
                rust_test_environment: false,
                build_workspace: false,
                test_argv: tsr_corpus::preparation::full_test_argv(),
            },
            &profile_dir,
        )
        .unwrap();
        let preparation =
            tsr_corpus::preparation::Preparation::load(&profile_dir.join("preparation.json"))
                .unwrap();
        task.preparation_id = Some(id.clone());
        crate::write_tasks(&manifest, &[task]).unwrap();
        let out = dir.path().join("prepared-out");
        assert_eq!(
            qualify(
                &manifest,
                &out,
                &Inert {
                    preparation: Some(preparation),
                    sabotage: 0,
                    calls: std::cell::Cell::new(0)
                }
            )
            .unwrap(),
            1
        );
        let exported =
            tsr_corpus::preparation::Preparation::load(&out.join("preparation.json")).unwrap();
        assert_eq!(exported.id(), id);
    }

    #[test]
    fn completed_groups_and_patch_scope_are_required() {
        let dir = tempfile::tempdir().unwrap();
        let patch=b"diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n";
        assert_eq!(patch_path(dir.path(), patch).unwrap(), "src/lib.rs");
        let mut extra = patch.to_vec();
        extra.extend_from_slice(b"diff --git a/target/extra.rs b/target/extra.rs\nnew file mode 100644\n--- /dev/null\n+++ b/target/extra.rs\n@@ -0,0 +1 @@\n+extra\n");
        assert!(patch_path(dir.path(), &extra).is_err());
        assert!(completed_suite(b"test x ... FAILED\n").is_err());
        assert!(completed_suite(b"running 1 test\ntest x ... FAILED\n").is_err());
        let group=b"running 1 test\ntest x ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
        let mut repeated = group.to_vec();
        repeated.extend_from_slice(group);
        assert_eq!(completed_suite(&repeated).unwrap(), 2);
    }
}
