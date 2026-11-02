//! Bounded synthetic snapshot/transport for legacy orchestration fixtures. Never executes source.
#![allow(dead_code)]
use std::{path::Path, process::Command};
pub fn repair_manifest(path: &Path) {
    let mut tasks = tsr_batch::task::read_tasks(path).unwrap();
    let repo = path.parent().unwrap().join("bounded-repo");
    std::fs::create_dir_all(&repo).unwrap();
    let mut files = std::collections::BTreeMap::<String, String>::new();
    let mut seen = std::collections::BTreeSet::new();
    for task in &tasks {
        for rel in &task.oracle_test_files {
            files.entry(rel.clone()).or_default();
        }
        for name in task.failing_tests() {
            let rel = task.oracle_test_files.first().unwrap();
            if !seen.insert((rel.clone(), name.clone())) {
                continue;
            }
            let source = files.entry(rel.clone()).or_default();
            let parts: Vec<_> = name.split("::").collect();
            // Separate modules per task fixture name; fixture inputs do not duplicate identities.
            for m in &parts[..parts.len() - 1] {
                source.push_str(&format!("mod {m} {{ "));
            }
            source.push_str(&format!(
                "#[test] fn {}() {{ assert_eq!(2,2); }}",
                parts.last().unwrap()
            ));
            for _ in &parts[..parts.len() - 1] {
                source.push('}');
            }
            source.push_str(&format!("\n// M3_SYNTHETIC_TEST_NAME:{name}\n"));
        }
    }
    for (rel, source) in files {
        let p = repo.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, source).unwrap();
    }
    let git = |args: &[&str]| {
        let o = Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "-qm",
        "bounded synthetic snapshot",
    ]);
    let tree = git(&["rev-parse", "HEAD^{tree}"]);
    for task in &mut tasks {
        task.repo = repo.canonicalize().unwrap().display().to_string();
        task.parent_snapshot = Some(tree.clone());
        task.mutated_snapshot = None;
        task.base_snapshot = None;
        task.mutation_diff = None;
    }
    std::fs::write(
        path,
        tasks
            .iter()
            .map(|t| serde_json::to_string(t).unwrap() + "\n")
            .collect::<String>(),
    )
    .unwrap();
}
pub fn command(bin: impl AsRef<std::ffi::OsStr>, root: &Path) -> Command {
    let dir = root.join("inert-bin");
    std::fs::create_dir_all(&dir).unwrap();
    let worker = dir.join("docker");
    std::fs::write(
        &worker,
        r#"#!/usr/bin/env python3
import sys,pathlib
args=sys.argv[1:]
if args[0]=='rm': sys.exit(0)
assert args[0]=='run' and '--network=none' in args
root=pathlib.Path(args[args.index('--workdir')+1])
names=set()
for p in root.rglob('*.rs'):
 if p.is_file() and not p.is_symlink():
  for line in p.read_text()[:100000].splitlines():
   if line.startswith('// M3_SYNTHETIC_TEST_NAME:'): names.add(line.split(':',1)[1])
for name in sorted(names):print('test '+name+' ... ok')
print('test result: ok.')
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&worker, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut cmd = Command::new(bin);
    cmd.env(
        "PATH",
        format!(
            "{}:{}",
            dir.display(),
            std::env::var("PATH").unwrap_or_default()
        ),
    )
    .env("TSR_M3_RUNTIME", "docker")
    .env(
        "TSR_M3_IMAGE",
        format!("synthetic@sha256:{}", "a".repeat(64)),
    )
    .env("TSR_M3_USER", "65534:65534");
    cmd
}
pub struct SyntheticRunner;
impl tsr_corpus::Runner for SyntheticRunner {
    fn provision(
        &self,
        _: &Path,
        repo: &Path,
        snapshot: &str,
        dest: &Path,
        _: &tsr_corpus::Limits,
    ) -> tsr_corpus::RunResult {
        tsr_corpus::trusted_provision(repo, snapshot, dest)
    }
    fn run(&self, _: &Path, argv: &[String], _: &tsr_corpus::Limits) -> tsr_corpus::RunResult {
        let name = argv
            .iter()
            .position(|s| s == "--")
            .and_then(|i| argv.get(i + 1));
        let text = name
            .map(|n| format!("test {n} ... ok\ntest result: ok.\n"))
            .unwrap_or_else(|| "test result: ok.\n".into());
        tsr_corpus::RunResult::from_output(0, text.as_bytes(), b"", 1)
    }
}

/// Synthetic arithmetic and mutant transport; candidate argv is inspected, never executed.
pub fn arithmetic_worker(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("docker");
    let script = format!(
        r#"#!/usr/bin/env python3
import json,os,pathlib,sys
args=sys.argv[1:]
if args[0]=='rm':sys.exit(0)
assert args[0]=='run' and '--network=none' in args and '--pull=never' in args
copy=pathlib.Path(args[args.index('--workdir')+1]).resolve()
assert copy.is_relative_to(pathlib.Path({allowed:?}).resolve())
for key in ['TSR_EXEC_LOG']:
 if os.environ.get(key):
  with open(os.environ[key],'a') as f:f.write(str(copy)+'\n')
if any('cargo mutants' in a or a=='mutants' for a in args):
 if os.environ.get('CARGO_MUTANTS_LOG'):
  with open(os.environ['CARGO_MUTANTS_LOG'],'a') as f:f.write(str(copy)+'\n')
 out=copy/'mutants.out';out.mkdir(exist_ok=True)
 mutant={{'name':'src/lib.rs:1:5: replace add -> i32 with 0','package':'demo','file':'src/lib.rs','function':{{'function_name':'add','return_type':'i32'}},'span':{{'start':{{'line':1,'column':5}},'end':{{'line':1,'column':5}}}},'replacement':'0','genre':'replace','diff':'--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 {{ a + b }}\n+pub fn add(a: i32, b: i32) -> i32 {{ 0 }}\n'}}
 if any('--list' in a for a in args):
  (out/'mutants.json').write_text(json.dumps([mutant]));sys.exit(0)
 (out/'log').mkdir(exist_ok=True)
 (out/'log/mutant.log').write_text('test adds ... FAILED\n')
 (out/'log/baseline.log').write_text('test adds ... ok\n')
 (out/'outcomes.json').write_text(json.dumps({{'outcomes':[{{'scenario':'Baseline','summary':'Success','log_path':'log/baseline.log'}},{{'scenario':{{'Mutant':mutant}},'summary':'CaughtMutant','log_path':'log/mutant.log'}}],'total_mutants':1,'caught':1,'missed':0,'timeout':0,'unviable':0,'success':0}}));sys.exit(2)
source=(copy/'src/lib.rs').read_text()[:10000]
passed='a + b' in source
(copy/'.tsr-suite-report.json').write_text(json.dumps({{'tests':[{{'name':'adds','passed':passed}}]}}))
print('test adds ... '+('ok' if passed else 'FAILED'))
print('test result: '+('ok.' if passed else 'FAILED.'))
sys.exit(0 if passed else 101)
"#,
        allowed = dir.parent().unwrap().display().to_string()
    );
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}
