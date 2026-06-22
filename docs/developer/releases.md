# Releases

Release process, GitHub Actions packaging, and Tauri updater setup for C4D Plugin Compiler.

## Overview

The release flow provides:

- Windows release builds through GitHub Actions.
- MSI and NSIS installers uploaded to a GitHub Release draft.
- Signed Tauri updater artifacts.
- `latest.json` uploaded to the latest GitHub Release for in-app updates.

## Current Configuration

- Repository: `DunHouGo/C4D-Plugin-Complier`
- Updater endpoint: `https://github.com/DunHouGo/C4D-Plugin-Complier/releases/latest/download/latest.json`
- Workflow: `.github/workflows/release.yml`
- Bundle artifacts: `msi,nsis`
- Release mode: draft release, published manually after review

## Signing Keys

The local updater key was generated with:

```bash
vp exec tauri signer generate --ci --write-keys C:\Users\DunHou\.tauri\c4d-plugin-compiler-updater.key
```

The private key stays outside the repository:

```text
C:\Users\DunHou\.tauri\c4d-plugin-compiler-updater.key
```

The public key is stored in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.

## GitHub Secrets

Add these in GitHub repository settings:

```text
Settings -> Secrets and variables -> Actions -> New repository secret
```

Required:

- `TAURI_SIGNING_PRIVATE_KEY`: the full contents of `C:\Users\DunHou\.tauri\c4d-plugin-compiler-updater.key`

Optional:

- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: only needed if the signing key was generated with a password

The current generated key has no password, so the password secret can be omitted.

## Release Process

Update versions before tagging:

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Run a local check:

```bash
vpr tauri:check
```

Create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will:

- Install dependencies with `npm ci`.
- Run `vpr typecheck`.
- Build Windows MSI and NSIS installers through `tauri-apps/tauri-action`.
- Sign updater artifacts with `TAURI_SIGNING_PRIVATE_KEY`.
- Create a draft GitHub Release.
- Upload installers, signatures, and `latest.json`.

After the workflow succeeds, review and publish the draft release manually on GitHub.

## Manual Workflow Dispatch

The release workflow can also be started manually from GitHub Actions.

Use a version input that starts with `v`, for example:

```text
v0.1.0
```

## Auto-Update Behavior

The app checks for updates shortly after startup and can also check from the app menu. Tauri verifies updater downloads with the public key in `tauri.conf.json`; if the signature does not match, the update is rejected.

## Troubleshooting

| Issue | Fix |
| ---- | --- |
| Workflow does not start | Make sure the pushed tag starts with `v` |
| Signing fails | Verify `TAURI_SIGNING_PRIVATE_KEY` contains the full private key file contents |
| Updates are not detected | Confirm the latest GitHub Release is published and contains `latest.json` |
| Signature verification fails | Make sure the private key secret matches the public key in `tauri.conf.json` |
| Local check fails on `--check` | Use `vpr tauri:check`; it maps to `tauri build --no-bundle` for this CLI version |
