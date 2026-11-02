//! Owner-invoked, offline full-workspace closeout for the recorded RTS replay.
use std::{fs, path::PathBuf};
use tsr_corpus::{
    sealed::{Runtime, SealedRunner},
    Limits, Runner,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        PathBuf::from(std::env::args().nth(1).ok_or("run directory required")?).canonicalize()?;
    let source = root.join("source").canonicalize()?;
    let evidence_root = if let Some(attempt) = std::env::args().nth(2) {
        if attempt.is_empty()
            || !attempt
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("literal attempt name required".into());
        }
        let path = root.join(attempt);
        fs::create_dir(&path)?;
        path
    } else {
        root.clone()
    };
    let runner = SealedRunner::new(
        Runtime::Docker,
        "sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6".into(),
        "501:20".into(),
    )?
    .with_read_only_workspace()
    .with_build_scratch(root.join("build").canonicalize()?)?;
    for stage in ["build", "test"] {
        let mut command: Vec<String> = ["sh", "-c", "mkdir -m 700 /tmp/m3-runtime /tmp/m3-state /tmp/m3-home && export CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tsr-build CARGO_NET_OFFLINE=true HOME=/tmp/m3-home XDG_RUNTIME_DIR=/tmp/m3-runtime XDG_STATE_HOME=/tmp/m3-state RTS_DAEMON_BIN=/tsr-build/debug/rts-daemon && exec \"$@\"", "tsr-full-suite", "cargo", stage, "--locked", "--offline", "--workspace", "-j", "1", "--target-dir", "/tsr-build"].into_iter().map(str::to_owned).collect();
        if stage == "test" {
            command.extend(["--no-fail-fast", "--", "--test-threads=1"].map(str::to_owned));
        }
        let limits = Limits::new(1_200_000, 1 << 20);
        let result = runner.run(&source, &command, &limits);
        fs::write(
            evidence_root.join(format!("{stage}.stdout")),
            &result.stdout,
        )?;
        fs::write(
            evidence_root.join(format!("{stage}.stderr")),
            &result.stderr,
        )?;
        let evidence = serde_json::json!({"command":command,"limits":limits,"result":result});
        fs::write(
            evidence_root.join(format!("{stage}.json")),
            serde_json::to_vec_pretty(&evidence)?,
        )?;
        println!(
            "{stage}: exit={} duration_ms={}",
            result.exit, result.duration_ms
        );
        if result.exit != 0 {
            break;
        }
    }
    Ok(())
}
