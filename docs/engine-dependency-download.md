# Prepare a missing locked dependency

`scripts/engine-prepare-image.py` turns an existing Cargo admission into a new
local pinned image and a prepared scope file. It downloads only the public
crates.io archives named in the protected lockfile, verifies each archive's
SHA-256, and constructs a Cargo directory source. It does not compile or execute
application or dependency code. The Rust controller and sealed repair executor
are unchanged.

## Workflow

Initialize an admission using the original image. A missing dependency may cause
`project-prepare` to fail here; that does not alter the lockfile or enable network.

```sh
BASE=$(mktemp -d "$PWD/target/cargo-download-XXXXXX")
target/debug/tsr engine --directory "$BASE/admission" project-init \
  --spec examples/engine-cargo-download/scope.json \
  --source examples/engine-cargo-download
python3 scripts/engine-prepare-image.py \
  --directory "$BASE/admission" --out "$BASE/prepared"
target/debug/tsr engine --directory "$BASE/run" project-init \
  --spec "$BASE/prepared/prepared-spec.json"
target/debug/tsr engine --directory "$BASE/run" project-prepare
target/debug/tsr engine --directory "$BASE/run" request
```

The helper requires Python 3.11+ and the already installed Docker client/local
Unix-socket daemon. The base image must already exist locally. It creates a new
output directory and leaves the original admission untouched. The prepared spec
contains the original admitted contents with only the image ID changed. It does
not import later candidate edits into a new admission. Resume existing repairs
in their original engine directory; the prepared spec starts a new run.

After readiness succeeds, use the existing proposal/run/resume interfaces and
`project-export`. Missing dependency versions, invalid lockfiles and compilation
failures continue to fail closed through those interfaces. No automatic resolver,
lockfile rewriting, model call or repair launch is hidden inside preparation.

## Separation of downloading and execution

The helper reads the controller's saved `project-config`, not a candidate's Cargo
configuration. For each locked registry package it requests only
`https://static.crates.io/crates/NAME/NAME-VERSION.crate`. Download requests carry
no authentication, ignore proxy configuration, and refuse redirects. Only crate
names and versions are disclosed; project source, tests and lockfiles are not
uploaded. Crate bytes must match the existing lock checksum before extraction.

Extracted sources receive per-file checksums in Cargo's directory-source format.
See the official [Cargo source-replacement documentation](https://doc.rust-lang.org/cargo/reference/source-replacement.html)
and [registry download/index specification](https://doc.rust-lang.org/cargo/reference/registry-index.html).
These metadata files support Cargo's integrity checks; they are not independent
attestation against a malicious host. The archive-to-lock comparison occurs
before image construction.

The generated Dockerfile has only `FROM`, `COPY` and `ENV` instructions. Its build
context contains the verified dependency sources and fixed Cargo configuration,
not project code or host credentials. The build uses `--network=none`,
`--pull=false`, an empty Docker client configuration, the selected local Unix
socket, and the installed legacy builder. There are no `RUN` instructions,
credential helpers, secret mounts or dependency build scripts in preparation.
The helper does not restart Docker or change daemon/global security settings.

The derived image contains its own offline Cargo source replacement. The existing
`project-prepare` then verifies availability via sealed `cargo fetch --locked
--offline`; repair compilation/tests retain the existing network-disabled,
nonroot, read-only workspace and bounded temporary build output. Any dependency
build scripts or proc macros execute only inside that later sealed compilation.

## Observed validation

The fixture uses `itoa = "=1.0.17"` in editable source. The prior image's directory
source had only `itoa 1.0.18`, so its initial `project-prepare` returned exit 101
and `ready: false`. The 1.0.17 checksum was obtained from the official crates.io
index and then frozen in the protected fixture lockfile.

Preparation downloaded 15,905 bytes and verified SHA-256
`92ecc6618181def0457392ccd0ee51198e065e016d1d527a7ac1b6dc7c1f09d2`.
The new local image is
`sha256:789290074c4f9289a88ecacbacae7e72472856ff35de071ffe2301f243fc42e7`.
The base is the prior approved `sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.

[Preparation evidence](engine-dependency-preparation-evidence.json) records the
initial missing-version failure, verified package, image ID and build log.
[Repair evidence](engine-cargo-download-evidence.json) records successful offline
readiness, partial repair failure, restart with named feedback, corrective repair,
four passing tests, another restart without another proposal, and patch export.
The exported patch applied to a disposable original snapshot and matched all
admitted repaired files. Original source, manifests, lockfile and protected tests
remained unchanged.

Reproduce the transport regression after preparation:

```sh
TSR_ENGINE_TEST_IMAGE=$(cat "$BASE/prepared/image.id") \
TSR_ENGINE_PREPARED_SPEC="$BASE/prepared/prepared-spec.json" \
TSR_ENGINE_TEST_KEEP=1 python3 -m unittest \
  scripts.test_engine_run.EngineRunTests.test_downloaded_cargo_workspace_pause_resume_and_export -v
```

Checks actually run:

- Missing-version preparation on the original image: failed as expected.
- Official checksum-verified download and network-disabled local image build: passed.
- Actual Cargo repair/feedback/restart/protected-file/export test: passed.
- `python3 -m unittest scripts.test_engine_prepare_image -v`: seven tests passed,
  covering valid checksums, checksum mismatch, unsafe paths, links/special files,
  duplicate paths, forged checksum metadata, expansion/decompression limits,
  foreign sources/unsafe URL fields and redirects.
- Python compilation and `git diff --check`: passed. No Rust runtime files changed.

No paid provider calls occurred. The repair evidence's billing entries are
synthetic loopback fixtures. No push, publication, deployment or image upload
occurred; the image remains local.

## Remaining bounds

Preparation supports the current exact stable numeric versions and crates.io-only
lock inventory, up to 32 packages. Downloads are capped at 10 MiB per archive and
64 MiB total, with a 45-second checked deadline per download and 10-second socket
timeout. Extracted file contents are capped at 128 MiB total; decompressed tar data
is separately capped at 128 MiB per archive, with 10,000 entries per archive.
Links, special files and ambiguous paths are refused. A crate incompatible with
these limits needs a separately reviewed preparation path; limits are not relaxed
automatically. Image construction has a 120-second timeout. Partial output is
retained on failure; reruns require a fresh output directory. There is no shared
cache eviction, automatic cleanup, registry mirror, authentication or image push.

This fills the missing-dependency preparation gap for the bounded Cargo adapter.
It does not change oracle limitations or supply broader real engine outcome
evidence across the existing 151 admitted corpus tasks. M3 acceptance remains
separate from this synthetic capability demonstration.
