# Approved browser dependency setup — blocked at sandbox startup

The approved acquisition and thin build succeeded. The single authored capability
probe failed under unchanged isolation; this runtime path is stopped. This packet
is historical setup evidence, **not** a game-run admission or retry authority.

- Source preparation: `485a016b95bba269786cc10c0b0935dc6488cb4c`.
- Official ARM64 base: `mcr.microsoft.com/playwright@sha256:969a9e3655576186a7f3b3d85502f8c4041f4afce571df7aadba6384d93ee61f`.
- Resulting local image: `sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883` (`linux/arm64`, user `501:20`).
- `playwright@1.64.1` and `playwright-core@1.64.1`: package metadata and downloaded
  archives matched the previously pinned SHA-512 integrity values in `packages.json`.
- The five-file package-only build context used `npm ci --offline --ignore-scripts
  --omit=optional --no-audit --no-fund`, with browser download disabled. Docker build
  itself used `--network=none --pull=false`. No host package was installed.
- Archived text recipes are here; the verified tarballs/full build context remain
  locally at `target/browser-runtime-setup-v1/context`, not committed as binaries.

## Actual verification result

`probe-command.json` records the exact one-shot authored command. It mounted no
workspace and retained network-none, read-only filesystem, all capabilities dropped,
no-new-privileges, UID/GID 501:20, 256 PID / 2 GiB memory / two CPU limits, existing
256 MiB temporary filesystem, and `chromiumSandbox:true`. No custom seccomp profile,
host IPC, SYS_ADMIN, or `--no-sandbox` was used.

The probe returned **exit 2**. Chromium terminated with **SIGTRAP** and:

> No usable sandbox!

Browser launch failed before the authored page was opened. No render, browser harness
pass, game behavior or full browser/controller integration is claimed. The log's
suggested unsafe workaround was not followed. The log alone cannot distinguish the
underlying namespace/seccomp/setuid mechanism; that remains unproven.

Playwright reported process cleanup. A subsequent read-only lookup by the exact
container name found no remaining container (`cleanup.json`). There is no uncertain
probe effect and no redispatch.

Independent read-only review by `/root/parity_review` verified package integrity,
minimal build scope, preserved controls and the known failed outcome. It found no
basis to label the runtime compatible. The unrelated `docker system df` diagnostic
returned a daemon snapshot-not-found error; it was not a build or browser success
signal and no cleanup/security change was attempted to address it.

## Remaining boundary

A compatible sandboxed browser runtime is still required. Do not activate the paid
browser launcher or weaken controls to finish this check. A focused diagnosis of
sandbox availability or a separately reviewed compatible execution environment is
needed before retrying browser verification. No alternative runtime or security
change is approved by this evidence.

The remaining live transport/admission connection and actual end-to-end browser tests
are **not completed**: they remain gated on compatible runtime verification. Existing
local authored native/projection/checkpoint tests are preserved in the prior packet.
No model has generated a game and no playable artifact exists yet.

Campaign accounting is unchanged: **259 calls / 72 operations / $0.222740**.
This setup used one image pull, one thin build and one explicitly authorized authored
capability probe; zero paid calls, game runs, campaign operations or security changes.
