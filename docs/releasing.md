# Releasing

Releases use a `release/vMAJOR.MINOR.PATCH` branch.

## Prepare a release

1. Update `version` in `Cargo.toml` and `Cargo.lock` if needed.
2. Create and push a release branch, for example:

   ```bash
   git switch -c release/v0.3.0
   git push codeberg release/v0.3.0
   ```

3. `prepare-release.yml` validates the branch and Cargo version, checks the
   Codeberg and GitHub credentials, runs the Cargo package/publish dry-run, and
   commits the generated `CHANGELOG.md` to the release branch.
4. Open a pull request from the release branch into `main`.

The preflight checks authenticate against Codeberg and GitHub before the pull
request is merged. crates.io does not provide a safe read-only token probe, so
its token is verified by the real `cargo publish` step after merge; the dry-run
still validates packaging and dependencies before merge.

## Required secrets

Configure these secrets in the Codeberg repository:

- `CARGO_REGISTRY_TOKEN`: crates.io publishing token.
- `GH_RELEASE_TOKEN`: GitHub token with permission to create releases in
  `gbrennon/tmux-worktrees`.

The Forgejo-provided `${{ github.token }}` is used for Codeberg tags and
releases. Configure Codeberg Actions authorization so this built-in token can
write repository contents; Forgejo does not support the GitHub-style workflow
`permissions` field.

## Publish

After the release pull request is merged, `release.yml` publishes the root
crate, pushes the release tag, generates release notes, and creates releases
on both Codeberg and GitHub.
