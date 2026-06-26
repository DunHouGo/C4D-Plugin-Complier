# Releases

Release process, GitHub Actions packaging, and Tauri updater setup for C4D Plugin Compiler.

## Overview

The release flow provides:

- Windows and macOS release builds through GitHub Actions.
- Windows MSI/NSIS installers and macOS DMG/app bundles uploaded to a published GitHub Release.
- Signed Tauri updater artifacts.
- `latest.json` uploaded to the latest GitHub Release for in-app updates.

## Current Configuration

- Repository: `DunHouGo/C4D-Plugin-Complier`
- Updater endpoint: `https://github.com/DunHouGo/C4D-Plugin-Complier/releases/latest/download/latest.json`
- Workflow: `.github/workflows/release.yml`
- Trigger: pushed tags matching `v*`
- Manual test workflow: `.github/workflows/manual-build.yml`
- Windows bundle artifacts: `msi,nsis`
- macOS bundle artifacts: universal `dmg,app`
- Release mode: published release, created automatically after tag builds succeed

## Signing Keys

The local updater key was generated with:

```bash
npm run tauri -- signer generate --ci --write-keys C:\Users\DunHou\.tauri\c4d-plugin-compiler-updater.key
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

Required:

- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password for the current updater signing key

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
git tag v0.1.7
git push origin v0.1.7
```

GitHub Actions will:

- Install dependencies with `npm ci`.
- Synchronize `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` from the pushed `v*` tag before building, so updater manifests use the release tag version even if a local version bump was missed.
- Run `npm exec -- vp exec tsc --noEmit`.
- Build Windows MSI/NSIS installers and macOS universal DMG/app bundles through `tauri-apps/tauri-action`.
- Sign updater artifacts with `TAURI_SIGNING_PRIVATE_KEY`.
- Create a published GitHub Release.
- Upload installers, signatures, and `latest.json`.

After the workflow succeeds, the release is immediately visible on GitHub Releases and the assets can be downloaded.

## Manual Test Build

Use the `Manual Test Build` workflow from GitHub Actions when you want to verify the Windows and macOS build pipeline without creating a GitHub Release.

By default it runs no-bundle smoke builds and uploads only app binaries as workflow artifacts. Enable the `bundle` input only when installer or app bundle artifacts are needed for testing. Frontend `dist` is an internal build input and is not uploaded separately.

## Auto-Update Behavior

The app checks for updates shortly after startup and can also check from the app menu. Tauri verifies updater downloads with the public key in `tauri.conf.json`; if the signature does not match, the update is rejected.

## Troubleshooting

| Issue | Fix |
| ---- | --- |
| Workflow does not start | Make sure the pushed tag starts with `v` |
| Signing fails | Verify `TAURI_SIGNING_PRIVATE_KEY` contains the full private key file contents |
| Updates are not detected | Confirm the latest GitHub Release is published, contains `latest.json`, and that `latest.json.version` is greater than the installed app version |
| Signature verification fails | Make sure the private key secret matches the public key in `tauri.conf.json` |
| Local check fails on `--check` | Use `vpr tauri:check`; it maps to `tauri build --no-bundle` for this CLI version |
