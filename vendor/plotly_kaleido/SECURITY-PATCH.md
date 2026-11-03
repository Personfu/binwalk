# Compatible installer security patch

This is upstream `plotly_kaleido` 0.13.6, from the checksum-verified crates.io
archive (`217959d69c1ea5089428b7614e96a8ab5e5aaf0b473ad65f56afb1cc79d0d152`),
with version build metadata `+fllc.1`. `Cargo.toml.orig` preserves its upstream
manifest. The upstream library API and existing Plotly/Kaleido features remain.

The build dependency uses actual ZIP 9.3.0, addressing CVE-2026-115905,
CVE-2026-115906 and CVE-2026-115909. The installer now uses enclosed paths,
rejects symlinks and inconsistent filenames, refuses existing-file overwrite,
bounds extraction, and does not print archive names/comments. Downloading uses
the platform HTTPS client with checked exit status instead of installing an
unpinned `ruget` executable during a Cargo build.

Both the main and fuzzing manifests patch this parent crate; each owns its
lockfile. Regression fixtures contain inert text only, with temporary paths.

This does not upgrade the legacy Kaleido 0.2.1 browser runtime or attest to
installed Binwalk binaries. Those require separate renderer/runtime migration
and release rollout review. Firmware ZIP extraction uses Binwalk's own parser
and 7z; the ZIP dependency being patched here is plotting installer tooling.

Advisory: https://github.com/zip-rs/zip2/security/advisories/GHSA-m665-v7pm-7grj
