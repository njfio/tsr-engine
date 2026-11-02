//! Explicit M3 worker: no shell, provider credentials, event store or Docker socket
//! is mounted into evaluated code. Constructing a plan never contacts a runtime.
use crate::{Limits, RunResult, Runner};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Runtime {
    Docker,
    Podman,
}
impl Runtime {
    fn program(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }
}

pub struct SealedRunner {
    runtime: Runtime,
    image: String,
    user: String,
    preparation: Option<crate::preparation::Preparation>,
    read_only_workspace: bool,
    build_scratch: Option<PathBuf>,
}
impl SealedRunner {
    /// Owner-selected, preloaded image pinned to its digest, and numeric uid:gid.
    pub fn new(runtime: Runtime, image: String, user: String) -> Result<Self, String> {
        let valid = if let Some(digest) = image.strip_prefix("sha256:") {
            digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
        } else if let Some((name, digest)) = image.split_once("@sha256:") {
            !name.is_empty()
                && name.as_bytes()[0].is_ascii_alphanumeric()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._/-:".contains(&b))
                && digest.len() == 64
                && digest.bytes().all(|b| b.is_ascii_hexdigit())
        } else {
            false
        };
        if !valid {
            return Err(
                "worker image must be an immutable image ID or digest-pinned reference".into(),
            );
        }
        let ids: Vec<_> = user.split(':').collect();
        if ids.len() != 2
            || ids.iter().any(|id| {
                id.is_empty()
                    || !id.bytes().all(|b| b.is_ascii_digit())
                    || id.parse::<u32>().is_err()
            })
        {
            return Err("worker user must be numeric uid:gid".into());
        }
        Ok(Self {
            runtime,
            image,
            user,
            preparation: None,
            read_only_workspace: false,
            build_scratch: None,
        })
    }
    /// Keep evaluated program executions from modifying source, probes or artifacts.
    pub fn with_read_only_workspace(mut self) -> Self {
        self.read_only_workspace = true;
        self
    }
    /// Fresh controller-owned build output; never source or a shared cache.
    pub fn with_build_scratch(mut self, path: PathBuf) -> Result<Self, String> {
        if !path.is_absolute() || path.to_str().is_none_or(|p| p.contains([',', '\n', '\r'])) {
            return Err("invalid scratch mount".into());
        }
        self.build_scratch = Some(path);
        Ok(self)
    }
    pub fn with_preparation(
        mut self,
        preparation: crate::preparation::Preparation,
    ) -> Result<Self, String> {
        if preparation.spec().image != self.image {
            return Err("worker image differs from preparation".into());
        }
        self.preparation = Some(preparation);
        Ok(self)
    }
    /// The evaluated argv follows the image, never enters a host shell.
    pub fn plan(&self, copy: &Path, argv: &[String], name: &str) -> Result<Vec<String>, String> {
        if !copy.is_absolute()
            || copy.to_str().is_none_or(|p| p.contains([',', '\n', '\r']))
            || argv.is_empty()
        {
            return Err("worker needs an absolute mount path and nonempty argv".into());
        }
        if !name.starts_with("tsr-m3-")
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid worker name".into());
        }
        let path = copy.display().to_string();
        let mut result = vec![
            "run".into(),
            "--rm".into(),
            // Reap orphaned descendants from subprocess-heavy integration suites.
            "--init".into(),
            "--pull=never".into(),
            "--name".into(),
            name.into(),
            "--network=none".into(),
            "--read-only".into(),
            "--cap-drop=ALL".into(),
            "--security-opt=no-new-privileges".into(),
            "--pids-limit=256".into(),
            "--memory=2g".into(),
            "--cpus=2".into(),
            "--user".into(),
            self.user.clone(),
            // Rustdoc executes generated test harnesses from its temporary directory.
            // Execution stays inside this nonroot, networkless worker; no host mount is added.
            "--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m".into(),
            "--mount".into(),
            format!(
                "type=bind,src={path},dst={path}{}",
                if self.read_only_workspace {
                    ",readonly"
                } else {
                    ""
                }
            ),
            "--workdir".into(),
            path,
            "--entrypoint=".into(),
            self.image.clone(),
        ];
        if let Some(scratch) = &self.build_scratch {
            let at = result.len() - 1; // Insert runtime flags before the immutable image.
            result.splice(
                at..at,
                [
                    "--mount".into(),
                    format!("type=bind,src={},dst=/tsr-build", scratch.display()),
                ],
            );
        }
        if let Some(prep) = &self.preparation {
            let mut command = argv.to_vec();
            if argv.len() >= 2 && argv[0] == "cargo" && argv[1] == "test" {
                command = prep.spec().test_argv.clone();
                command.extend_from_slice(&argv[2..]);
                if !command.iter().any(|a| a == "--") {
                    command.push("--".into());
                }
                command.push("--test-threads=1".into());
            }
            if prep.spec().rust_test_environment {
                // Fixed worker-local startup only; candidate argv remains separate positional arguments.
                result.extend(["sh".into(),"-c".into(),"mkdir -m 700 /tmp/m3-runtime /tmp/m3-state /tmp/m3-home && export CARGO_INCREMENTAL=0 HOME=/tmp/m3-home XDG_RUNTIME_DIR=/tmp/m3-runtime XDG_STATE_HOME=/tmp/m3-state RTS_DAEMON_BIN=\"$PWD/target/debug/rts-daemon\" && exec \"$@\"".into(),"tsr-worker".into()]);
            }
            result.extend(command);
        } else {
            if self.build_scratch.is_some() {
                // Nested Cargo/rustdoc and subprocess tests need the same admitted
                // writable scratch and private runtime directories as their parent.
                // No candidate text enters this fixed shell program.
                result.extend(["sh".into(), "-c".into(),
                    "mkdir -m 700 /tmp/tsr-runtime /tmp/tsr-state && export CARGO_TARGET_DIR=/tsr-build CARGO_BUILD_JOBS=1 XDG_RUNTIME_DIR=/tmp/tsr-runtime XDG_STATE_HOME=/tmp/tsr-state && exec \"$@\"".into(),
                    "tsr-worker".into()]);
            }
            result.extend_from_slice(argv);
        }
        Ok(result)
    }
    fn remove(&self, program: &Path, name: &str) {
        // Only the name minted for this run is eligible for cleanup. Bounded client wait.
        if let Ok(mut child) = Command::new(program)
            .args(["rm", "--force", name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
fn scratch_io_error(stage: &str, error: &std::io::Error) -> String {
    format!(
        "scratch_{stage}_read_error:kind={:?}:os={:?}",
        error.kind(),
        error.raw_os_error()
    )
}
fn scratch_failure(root: &Path) -> Option<String> {
    scratch_scan(root, |_| {})
}
fn scratch_scan(root: &Path, mut before_read: impl FnMut(&Path)) -> Option<String> {
    let mut todo = vec![root.to_path_buf()];
    let mut bytes = 0u64;
    let mut entries = 0usize;
    while let Some(dir) = todo.pop() {
        before_read(&dir);
        let items = match std::fs::read_dir(&dir) {
            Ok(items) => items,
            // Build tools rename/remove temporary subdirectories while compiling.
            // A vanished child contributes no remaining allocation. The root must exist.
            Err(error) if dir != root && error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Some(scratch_io_error("directory", &error)),
        };
        for item in items {
            let item = match item {
                Ok(item) => item,
                Err(error) => return Some(scratch_io_error("entry", &error)),
            };
            let meta = match item.path().symlink_metadata() {
                Ok(meta) => meta,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Some(scratch_io_error("metadata", &error)),
            };
            entries += 1;
            if entries > 100000 || meta.file_type().is_symlink() {
                return Some("scratch_entry_limit_or_symlink".into());
            }
            if meta.is_dir() {
                todo.push(item.path());
            } else if meta.is_file() {
                bytes = bytes.saturating_add(meta.len());
            } else {
                return Some("scratch_nonregular_entry".into());
            }
            if bytes > 8 * 1024 * 1024 * 1024 {
                return Some("scratch_byte_limit".into());
            }
        }
    }
    None
}
#[cfg(test)]
fn scratch_within_budget(root: &Path) -> bool {
    scratch_failure(root).is_none()
}
struct Capture {
    done: std::sync::mpsc::Receiver<bool>,
    bytes: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}
impl Capture {
    fn finish(self, wait: Duration) -> (Vec<u8>, bool) {
        let complete = self.done.recv_timeout(wait).unwrap_or(false);
        (self.bytes.lock().expect("capture lock").clone(), complete)
    }
}
fn capture(mut stream: impl Read + Send + 'static, max: usize) -> Capture {
    let (send, done) = std::sync::mpsc::channel();
    let bytes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let output = bytes.clone();
    std::thread::spawn(move || {
        let mut complete = true;
        let mut buffer = [0; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let mut captured = output.lock().expect("capture lock");
                    let keep = n.min(max.saturating_sub(captured.len()));
                    captured.extend_from_slice(&buffer[..keep]);
                    if keep < n {
                        complete = false;
                    }
                }
                Err(_) => {
                    complete = false;
                    break;
                }
            }
        }
        let _ = send.send(complete);
    });
    Capture { done, bytes }
}
impl SealedRunner {
    fn execute(&self, copy: &Path, argv: &[String], limits: &Limits, program: &Path) -> RunResult {
        let started = Instant::now();
        let fail = |message: String| {
            let mut result = RunResult::from_output(
                125,
                b"",
                message.as_bytes(),
                started.elapsed().as_millis() as u64,
            );
            result.execution = Some(crate::ExecutionDiagnostics {
                version: 1,
                termination: "pre_spawn_failure".into(),
                runtime_exit: None,
                runtime_signal: None,
                stdout_complete: true,
                stderr_complete: true,
                worker: String::new(),
            });
            result
        };
        if limits.wall_clock_ms == 0 || limits.max_output_bytes == 0 {
            return fail("zero execution limit".into());
        }
        let copy: PathBuf = match copy.canonicalize() {
            Ok(path) => path,
            Err(error) => return fail(error.to_string()),
        };
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = format!(
            "{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let name = format!("tsr-m3-{nonce}");
        let plan = match self.plan(&copy, argv, &name) {
            Ok(plan) => plan,
            Err(error) => return fail(error),
        };
        let mut child = match Command::new(program)
            .args(plan)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => return fail(error.to_string()),
        };
        let stdout = capture(child.stdout.take().expect("piped"), limits.max_output_bytes);
        let stderr = capture(child.stderr.take().expect("piped"), limits.max_output_bytes);
        let mut last_size_check = Instant::now();
        let mut termination = "runtime_exit".to_string();
        let mut runtime_exit = None;
        let mut runtime_signal = None;
        let exit = loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    runtime_exit = status.code();
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        runtime_signal = status.signal();
                    }
                    if runtime_signal.is_some() {
                        termination = "runtime_signal".into();
                    }
                    break status.code().unwrap_or(-1);
                }
                Err(_) => {
                    termination = "runtime_wait_error".into();
                    let _ = child.kill();
                    if let Ok(status) = child.wait() {
                        runtime_exit = status.code();
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            runtime_signal = status.signal();
                        }
                    }
                    self.remove(program, &name);
                    break 125;
                }
                Ok(None) => {}
            }
            if last_size_check.elapsed() >= Duration::from_secs(1) {
                last_size_check = Instant::now();
                if let Some(reason) = self.build_scratch.as_ref().and_then(|p| scratch_failure(p)) {
                    termination = reason;
                    let _ = child.kill();
                    if let Ok(status) = child.wait() {
                        runtime_exit = status.code();
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            runtime_signal = status.signal();
                        }
                    }
                    self.remove(program, &name);
                    break 125;
                }
            }
            if started.elapsed() >= Duration::from_millis(limits.wall_clock_ms) {
                termination = "wall_timeout".into();
                let _ = child.kill();
                if let Ok(status) = child.wait() {
                    runtime_exit = status.code();
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        runtime_signal = status.signal();
                    }
                }
                self.remove(program, &name);
                break 124;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let (out, out_complete) = stdout.finish(Duration::from_secs(1));
        let (err, err_complete) = stderr.finish(Duration::from_secs(1));
        let exit = if exit != 124 && (!out_complete || !err_complete) {
            125
        } else {
            exit
        };
        let mut result =
            RunResult::from_output(exit, &out, &err, started.elapsed().as_millis() as u64);
        result.execution = Some(crate::ExecutionDiagnostics {
            version: 1,
            termination,
            runtime_exit,
            runtime_signal,
            stdout_complete: out_complete,
            stderr_complete: err_complete,
            worker: name,
        });
        result
    }
}
impl Runner for SealedRunner {
    fn prepared_lock(&self, snapshot: &str) -> Option<&[u8]> {
        self.preparation.as_ref().and_then(|p| p.lock(snapshot))
    }
    fn preparation_id(&self) -> Option<&str> {
        self.preparation.as_ref().map(|p| p.id())
    }
    fn repository_identity(&self) -> Option<&str> {
        self.preparation
            .as_ref()
            .map(|p| p.spec().repository_identity.as_str())
    }
    fn prepare(
        &self,
        repo: &Path,
        snapshot: &str,
        dest: &Path,
        evidence: &Path,
    ) -> Result<(), String> {
        if let Some(p) = &self.preparation {
            let receipt = p.apply(repo, snapshot, dest)?;
            p.export(evidence)?;
            crate::mine::write_cas(&evidence.join("cas"), &crate::digest(&receipt), &receipt)
                .map_err(|e| e.to_string())?;
            if p.spec().build_workspace {
                let argv = [
                    "cargo",
                    "build",
                    "--locked",
                    "--offline",
                    "--workspace",
                    "-j",
                    "1",
                ]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>();
                let result = self.execute(
                    dest,
                    &argv,
                    &Limits::default(),
                    Path::new(self.runtime.program()),
                );
                crate::persist_run(evidence, snapshot, &argv, &result, Some(p.id()))?;
                if !result.succeeded() {
                    return Err(format!(
                        "prepared workspace build failed (exit {}), captured in runs.jsonl",
                        result.exit
                    ));
                }
            }
        }
        Ok(())
    }

    fn provision(
        &self,
        _root: &Path,
        repo: &Path,
        snapshot: &str,
        dest: &Path,
        _limits: &Limits,
    ) -> RunResult {
        crate::trusted_provision(repo, snapshot, dest)
    }
    fn run(&self, copy: &Path, argv: &[String], limits: &Limits) -> RunResult {
        self.execute(copy, argv, limits, Path::new(self.runtime.program()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn worker() -> SealedRunner {
        SealedRunner::new(
            Runtime::Docker,
            format!("rust@sha256:{}", "a".repeat(64)),
            "501:20".into(),
        )
        .unwrap()
    }
    #[test]
    fn private_build_mount_preserves_read_only_source() {
        let args = worker()
            .with_read_only_workspace()
            .with_build_scratch(PathBuf::from("/tmp/private-build"))
            .unwrap()
            .plan(
                Path::new("/work/source"),
                &["cargo".into()],
                "tsr-m3-scratch",
            )
            .unwrap();
        assert!(args.contains(&"type=bind,src=/work/source,dst=/work/source,readonly".into()));
        let mount = args
            .iter()
            .position(|a| a == "type=bind,src=/tmp/private-build,dst=/tsr-build")
            .unwrap();
        let image = args
            .iter()
            .position(|a| a.starts_with("rust@sha256:"))
            .unwrap();
        assert!(mount < image);
        assert!(worker()
            .with_build_scratch(PathBuf::from("relative"))
            .is_err());
    }
    #[cfg(unix)]
    #[test]
    fn scratch_budget_rejects_symlinks_and_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scratch_within_budget(dir.path()));
        let file = std::fs::File::create(dir.path().join("large")).unwrap();
        file.set_len(8 * 1024 * 1024 * 1024 + 1).unwrap();
        assert!(!scratch_within_budget(dir.path()));
        file.set_len(0).unwrap();
        std::os::unix::fs::symlink("/outside", dir.path().join("link")).unwrap();
        assert!(!scratch_within_budget(dir.path()));
    }
    #[test]
    fn read_only_probe_mount_cannot_change_workspace() {
        let runner = SealedRunner::new(
            Runtime::Docker,
            format!("sha256:{}", "a".repeat(64)),
            "501:20".into(),
        )
        .unwrap()
        .with_read_only_workspace();
        let args = runner
            .plan(
                Path::new("/tmp/owned-fixture"),
                &["./fixture-probe".into()],
                "tsr-m3-probe",
            )
            .unwrap();
        assert!(args
            .iter()
            .any(|a| a == "type=bind,src=/tmp/owned-fixture,dst=/tmp/owned-fixture,readonly"));
        assert_eq!(args.iter().filter(|a| a.as_str() == "--mount").count(), 1);
    }
    #[test]
    fn worker_plan_seals_and_does_not_interpret_candidate_argv() {
        let args = vec!["cargo".into(), "test".into(), "literal; no shell".into()];
        let plan = worker()
            .plan(Path::new("/work/copy"), &args, "tsr-m3-test")
            .unwrap();
        for flag in [
            "--network=none",
            "--init",
            "--read-only",
            "--pull=never",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m",
        ] {
            assert!(plan.contains(&flag.into()));
        }
        assert_eq!(&plan[plan.len() - 3..], &args);
        assert_eq!(plan.iter().filter(|v| v.as_str() == "--mount").count(), 1);
        assert!(!plan.iter().any(|v| v.contains("docker.sock")));
    }
    #[test]
    fn unsafe_configuration_is_refused_without_execution() {
        assert!(SealedRunner::new(Runtime::Docker, "rust:latest".into(), "0:0".into()).is_err());
        assert!(worker()
            .plan(
                Path::new("/work/copy,other"),
                &["true".into()],
                "tsr-m3-test"
            )
            .is_err());
        assert!(worker()
            .plan(Path::new("/work"), &["true".into()], "other-container")
            .is_err());
    }
    #[cfg(unix)]
    #[test]
    fn timeout_requests_cleanup_of_only_its_own_worker() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("inert-runtime");
        let marker = dir.path().join("cleanup");
        let script=format!("#!/bin/sh\nif [ \"$1\" = rm ]; then printf '%s\\n' \"$3\" > '{}'; exit 0; fi\nsleep 1\n", marker.display());
        std::fs::write(&program, script).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let result = worker().execute(
            dir.path(),
            &["inert-candidate-name".into()],
            &Limits::new(20, 128),
            &program,
        );
        assert_eq!(result.exit, 124);
        assert_eq!(
            result.execution.as_ref().unwrap().termination,
            "wall_timeout"
        );
        let name = std::fs::read_to_string(marker).unwrap();
        assert!(name.trim().starts_with("tsr-m3-"));
        assert!(name
            .trim()
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-'));
        assert!(result.duration_ms < 3000);
    }
    #[test]
    #[ignore = "requires explicitly approved Docker access and preloaded image"]
    fn real_worker_bounded_smoke() {
        let image = std::env::var("TSR_M3_SMOKE_IMAGE").expect("explicit smoke image");
        let runner = SealedRunner::new(Runtime::Docker, image, "65534:65534".into()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
        }
        // Fixed harmless commands only; no evaluated source, secrets or network probes.
        let run = |script: &str, ms, bytes| {
            runner.run(
                dir.path(),
                &["/bin/sh".into(), "-c".into(), script.into()],
                &Limits::new(ms, bytes),
            )
        };
        let success = run("test ! -e /var/run/docker.sock && test ! -w / && printf fixture > marker && printf sealed-ok", 10000, 128);
        assert_eq!(success.exit, 0, "{success:?}");
        assert_eq!(
            std::fs::read(dir.path().join("marker")).unwrap(),
            b"fixture"
        );
        assert_eq!(run("printf 0123456789; exit 101", 10000, 4).exit, 125);
        assert_eq!(run("sleep 10", 100, 128).exit, 124);
    }
    #[cfg(unix)]
    #[test]
    fn termination_receipts_distinguish_exit_signal_and_partial_output() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("inert-runtime");
        for (body, exit, raw, signal, complete) in [
            ("printf partial; exit 101", 101, Some(101), None, true),
            ("printf 123456789; exit 0", 125, Some(0), None, false),
            ("printf partial; kill -TERM $$", -1, None, Some(15), true),
            ("exit 125", 125, Some(125), None, true),
        ] {
            std::fs::write(&program, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
            let result = worker().execute(
                dir.path(),
                &["inert".into()],
                &Limits::new(1000, 7),
                &program,
            );
            assert_eq!(result.exit, exit);
            let d = result.execution.as_ref().unwrap();
            assert_eq!(d.runtime_exit, raw);
            assert_eq!(d.runtime_signal, signal);
            assert_eq!(d.stdout_complete, complete);
            assert!(result.stdout.len() <= 7);
            let saved = serde_json::to_vec(&result).unwrap();
            let replay: RunResult = serde_json::from_slice(&saved).unwrap();
            assert_eq!(replay.execution, result.execution);
            assert_eq!(replay.duration_ms, result.duration_ms);
        }
        let fail = worker().execute(
            dir.path(),
            &["inert".into()],
            &Limits::new(1000, 7),
            &dir.path().join("missing"),
        );
        assert_eq!(fail.execution.unwrap().termination, "pre_spawn_failure");
        let legacy: RunResult = serde_json::from_value(serde_json::json!({"exit":125,"stdout_digest":"x","stderr_digest":"y","duration_ms":42})).unwrap();
        assert!(legacy.execution.is_none());
        assert_eq!(legacy.duration_ms, 42);
    }
    #[test]
    fn incomplete_capture_retains_already_observed_prefix() {
        struct Delayed {
            emitted: bool,
            ready: std::sync::mpsc::Sender<()>,
            release: std::sync::mpsc::Receiver<()>,
        }
        impl Read for Delayed {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                if !self.emitted {
                    self.emitted = true;
                    buffer[..7].copy_from_slice(b"partial");
                    return Ok(7);
                }
                self.ready.send(()).unwrap();
                let _ = self.release.recv();
                Ok(0)
            }
        }
        let (ready, reached) = std::sync::mpsc::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let output = capture(
            Delayed {
                emitted: false,
                ready,
                release: wait,
            },
            8,
        );
        reached.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(
            output.finish(Duration::from_millis(1)),
            (b"partial".to_vec(), false)
        );
        release.send(()).unwrap();
    }
    #[test]
    fn scratch_scan_tolerates_only_disappeared_children() {
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("temporary");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("artifact"), b"bounded").unwrap();
        assert_eq!(
            scratch_scan(dir.path(), |next| {
                if next == child {
                    std::fs::remove_dir_all(&child).unwrap();
                }
            }),
            None
        );
        let missing = dir.path().join("missing-root");
        assert!(scratch_failure(&missing).unwrap().contains("kind=NotFound"));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("missing-target", dir.path().join("link")).unwrap();
            assert_eq!(
                scratch_failure(dir.path()),
                Some("scratch_entry_limit_or_symlink".into())
            );
        }
    }
    #[test]
    fn scratch_environment_is_private_and_argv_stays_literal() {
        let runner = worker()
            .with_read_only_workspace()
            .with_build_scratch(PathBuf::from("/scratch"))
            .unwrap();
        let argv = vec!["cargo".into(), "test".into(), "literal; not shell".into()];
        let plan = runner
            .plan(Path::new("/source"), &argv, "tsr-m3-test")
            .unwrap();
        assert_eq!(&plan[plan.len() - argv.len()..], &argv);
        assert!(plan
            .iter()
            .any(|s| s.contains("CARGO_TARGET_DIR=/tsr-build")
                && s.contains("mkdir -m 700")
                && s.contains("XDG_RUNTIME_DIR=/tmp/tsr-runtime")));
        assert!(plan
            .iter()
            .any(|s| s == "type=bind,src=/source,dst=/source,readonly"));
        assert!(plan.contains(&"--network=none".into()));
        assert!(plan.contains(&"--memory=2g".into()));
        assert_eq!(plan.iter().filter(|s| *s == "--mount").count(), 2);
        let plain = worker()
            .plan(Path::new("/source"), &argv, "tsr-m3-test")
            .unwrap();
        assert!(!plain.iter().any(|s| s.contains("XDG_RUNTIME_DIR")));
    }
    #[test]
    fn output_is_bounded_and_truncation_is_explicit() {
        let (bytes, complete) =
            capture(std::io::Cursor::new(vec![b'x'; 1024]), 8).finish(Duration::from_secs(1));
        assert_eq!(bytes.len(), 8);
        assert!(!complete);
    }
}
