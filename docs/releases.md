# CI and releases

CI runs on pushes to `main`, pull requests, manual dispatch, and as a reusable
release check. It checks formatting, strict Clippy, offline tests on Linux/macOS,
the declared Rust 1.88 minimum, and compilation of the actual Cargo
package. Package auditing checks the file allowlist, original license and example
configuration; Google credentials are never needed in CI.

Action dependencies are pinned to commit SHAs. Dependabot checks Cargo and Actions
dependencies weekly. Live Google tests remain opt-in because they are billable
and the local test key has an IP restriction.

## GitHub release

1. Update the Cargo version, lockfile, changelog and `.github/RELEASE_NOTES.md`.
2. Commit and push the release changes; wait for CI to pass.
3. Create an annotated tag matching the Cargo version, e.g. `v0.1.0`, and push it.
4. The Release workflow reruns CI on the exact tag, creates an unpublished draft,
   builds the three native executable targets, and smoke-tests version/discovery
   without credentials. Each build uploads its archive and checksum to the draft.
   After every build succeeds, the workflow downloads and verifies all archives,
   adds `SHA256SUMS`, and publishes the release.

The release build uses Rust 1.94.1 and `Cargo.lock`. Linux x86_64 binaries are built
on Ubuntu 22.04 for glibc 2.35 compatibility. The other targets are macOS Apple
Silicon and macOS Intel. Each archive includes documentation,
configuration example and licenses. macOS code signing is not configured.

Draft assets are used for staging, so releases do not depend on the account's
Actions artifact storage quota. A failed build leaves the release unpublished.
To retry, rerun failed jobs, or manually dispatch the latest Release workflow on
`main` with the existing source tag as input. This lets workflow fixes release an
unchanged source tag. Tags must match the Cargo version. Completed releases are
not overwritten by this workflow; create a new version for changed release files.

```sh
gh workflow run release.yml --ref main -f tag=v0.1.0
```

## crates.io publication

Publishing to crates.io is a separate manually dispatched workflow named
**Publish crate**. It requires an already-published GitHub release for the input
tag and checks out that exact tag, verifies packaging and then publishes the crate.

For the initial publish, create a short-lived crates.io API token with permission
to publish this crate, and set it as the `CARGO_REGISTRY_TOKEN` secret in the
`crates-io` GitHub environment (or repository). Keep the token out of chat and files
committed to Git. The workflow fails clearly if that secret has not been set.
Environment approvals may be configured by the repository owner if desired.

Dispatch from the repository Actions page, or use:

```sh
gh workflow run publish.yml -f tag=v0.1.0
```

Crate versions cannot be overwritten. The workflow does not run automatically on
tag pushes, and Google API keys must never be used as registry credentials.

After publishing, users can install with:

```sh
cargo install mcp-gmap-rs --locked
```
