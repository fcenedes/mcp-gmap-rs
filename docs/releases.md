# CI and releases

CI runs on pushes to `main`, pull requests, manual dispatch, and as a reusable
release check. It checks formatting, strict Clippy, offline tests on Linux/macOS/
Windows, the declared Rust 1.88 minimum, and compilation of the actual Cargo
package. Package auditing checks the file allowlist, original license and example
configuration; Google credentials are never needed in CI.

Action dependencies are pinned to commit SHAs. Dependabot checks Cargo and Actions
dependencies weekly. Live Google tests remain opt-in because they are billable
and the local test key has an IP restriction.

## GitHub release

1. Update the Cargo version, lockfile, changelog and `.github/RELEASE_NOTES.md`.
2. Commit and push the release changes; wait for CI to pass.
3. Create an annotated tag matching the Cargo version, e.g. `v0.1.0`, and push it.
4. The Release workflow reruns CI, builds the four native executable targets,
   smoke-tests version/discovery without credentials, and publishes their archives
   and SHA-256 checksums to a GitHub release.

The release build uses Rust 1.94.1 and `Cargo.lock`. Linux x86_64 binaries are built
on Ubuntu 22.04 for glibc 2.35 compatibility. The other targets are macOS Apple
Silicon, macOS Intel and Windows x86_64. Each archive includes documentation,
configuration example and licenses. macOS/Windows code signing is not configured.

The GitHub release is created only after every platform build succeeds. To retry
an interrupted release, rerun failed jobs or manually dispatch Release on the
existing tag. Tags must match the version; dispatching on a branch fails the gate.
An existing release receives rebuilt assets when the workflow is explicitly rerun.

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
