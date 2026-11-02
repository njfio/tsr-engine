//! Explicit owner configuration for the M3 networkless worker.
//! No credentials are read or forwarded. Configuration never probes the daemon.
use tsr_corpus::sealed::{Runtime, SealedRunner};
pub fn configured() -> Result<SealedRunner, String> {
    let get = |name| {
        std::env::var(name).map_err(|_| format!("missing {name}; configure an approved preloaded digest-pinned M3 worker image and numeric uid:gid"))
    };
    let runtime = match get("TSR_M3_RUNTIME")?.as_str() {
        "docker" => Runtime::Docker,
        "podman" => Runtime::Podman,
        _ => return Err("TSR_M3_RUNTIME must be docker or podman".into()),
    };
    let runner = SealedRunner::new(runtime, get("TSR_M3_IMAGE")?, get("TSR_M3_USER")?)?;
    match std::env::var("TSR_M3_PREPARATION") {
        Ok(path) => runner.with_preparation(tsr_corpus::preparation::Preparation::load(
            std::path::Path::new(&path),
        )?),
        Err(std::env::VarError::NotPresent) => Ok(runner),
        Err(_) => Err("invalid TSR_M3_PREPARATION".into()),
    }
}
