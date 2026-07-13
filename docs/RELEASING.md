# Releasing Layered Graphics

Layered Graphics uses one version across its Cargo workspace, npm packages, site, CLI binaries, and GitHub release. Releases are always prepared on a pull request and published only through a manually dispatched, approval-capable workflow.

## Published artifacts

| Artifact | Destination |
| --- | --- |
| `@layered-graphics/core` | npm |
| `@layered-graphics/browser` | npm |
| `layered-graphics` | crates.io |
| `layered-graphics-cli` (`lg`) | crates.io |
| Linux x64 CLI | GitHub Release `.tar.gz` |
| macOS x64 and ARM64 CLI | GitHub Release `.tar.gz` |
| Windows x64 CLI | GitHub Release `.zip` |
| Checksums, schemas, and editable examples | GitHub Release assets |

The native Node package and its Rust binding crate are intentionally private until per-platform N-API distribution is implemented.

## Prepare a version bump

Run the **Prepare release** workflow from GitHub Actions and choose `prerelease`, `patch`, `minor`, or `major`. A prerelease bump also accepts an identifier such as `alpha`.

The workflow runs `scripts/prepare-release.mjs`, which updates:

- the root, core, browser, Node, and site npm manifests;
- the Cargo workspace version and every internal version constraint;
- `Cargo.lock` and `pnpm-lock.yaml`; and
- the changelog release heading.

It pushes `release/v<version>`, opens a release PR, and explicitly dispatches CI for that protected-branch candidate. The same operation is available locally:

```bash
pnpm release:prepare prerelease alpha
pnpm release:prepare patch
pnpm release:prepare 0.2.0-alpha.0
```

Review and merge the release PR. Do not edit only one package version by hand.

## Publish the merged version

Run the **Publish release** workflow and enter the exact version from `main`, without the `v` prefix. The workflow refuses to publish a different version.

Before any permanent upload, it builds every CLI target and packs both npm packages. The final environment-controlled job then:

1. creates a draft GitHub release and tag;
2. uploads binaries, schemas, examples, and `SHA256SUMS`;
3. publishes the Rust core before the dependent CLI crate;
4. publishes core before browser on npm; and
5. makes the GitHub release public only after registry publication succeeds.

Reruns are safe: already-published npm/crates.io versions are detected and skipped, draft assets are replaced, and published GitHub releases are not modified automatically.

Prerelease versions use their identifier as the npm dist-tag (`alpha`, `beta`, or similar). Stable versions use `latest`.

## First-publication credentials

Both registries require the first version to be published before trusted publishing can be attached. Add these secrets to the repository's `release` environment for the first run:

- `NPM_TOKEN` — granular npm token able to publish the two packages in the `@layered-graphics` organization.
- `CARGO_REGISTRY_TOKEN` — crates.io token able to create the two crates.

The environment is deliberately separate from ordinary CI and pull requests.

## Switch to trusted publishing after the first release

After both npm packages exist, configure a GitHub Actions trusted publisher for each package with:

- owner: `iamkaf`
- repository: `layeredgraphics`
- workflow: `publish-release.yml`
- environment: `release`

Then delete `NPM_TOKEN`. npm 11 on the GitHub-hosted runner automatically uses OIDC and emits provenance.

After both crates exist, configure the same repository, workflow, and environment as a trusted publisher on each crates.io package, then delete `CARGO_REGISTRY_TOKEN`. The workflow automatically uses `rust-lang/crates-io-auth-action` when that secret is absent.

## Failure policy

Registry versions are immutable. Never reuse a failed version. If an upload reached a registry but the workflow did not complete, rerun the same version first; the workflow skips completed uploads. If the artifact itself is defective, finish or abandon the draft GitHub release, prepare a new patch/prerelease version, and yank the affected crate version only when necessary.
