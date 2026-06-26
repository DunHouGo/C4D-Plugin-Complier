# C4D Plugin Compiler

[简体中文](README.zh-CN.md) | English

C4D Plugin Compiler is a desktop, build and packaging tool for Cinema 4D C++ plugins. It helps prepare Maxon C++ SDK sources, detect the local Windows build environment, run the official CMake preset workflow, and package compiled plugins into C4D-ready folders.

![C4D Plugin Compiler interface](images/img.jpg)

The app is built with Tauri 2, Rust, React, and TypeScript. It is based on the [DunHouGo/tauri-desktop-starter](https://github.com/DunHouGo/tauri-desktop-starter) template and extends it with Cinema 4D SDK discovery, build orchestration, packaging, and updater release workflows. The current workflow supports Windows and macOS builds for Cinema 4D 2024.4 and newer.

## Features

- Manage one shared SDK root for Cinema 4D 2024.4, 2025, and 2026+ SDKs.
- Detect local Cinema 4D installations and map them to matching Maxon C++ SDK versions.
- Download or reuse cached SDK archives when a required SDK is missing.
- Check CMake, Visual Studio 2022, Windows SDK, and SDK availability before building.
- Build C++ plugins through Maxon's official CMake preset workflow.
- Generate merged packages, per-version packages, and optional zip archives.
- Copy plugin resources so each output folder can be selected directly as a Cinema 4D plugin.
- Preview the output file tree before running a build.

## Requirements

- Windows or macOS
- Node.js 20+
- Rust stable
- CMake
- Visual Studio 2022 with MSVC C++ build tools and Windows SDK on Windows
- Xcode, Clang, and Python 3.8 on macOS
- Cinema 4D 2024.4 or newer when local SDK detection is needed

## Quick Start

```bash
vp install
vpr dev
```

For a release build:

```bash
vpr tauri build
```

For a build check without creating installers:

```bash
vpr tauri build --no-bundle
```

## Basic Workflow

1. Set **SDK Root** in the SDK Sources panel( Default is `../Documents/Maxon_SDK`).
2. Click **Auto Detect** or **Refresh** to detect local Cinema 4D installs and available SDKs.
3. Set **Plugin Root**, **Module**, **Package**, target C4D versions, configuration, package mode, and output folder.
4. Use **Output Preview** to confirm the folder layout.
5. Click **Build** to resolve SDKs, compile the module, and package the artifacts.

See [readme-en.md](readme-en.md) for the detailed user guide.

## Project Structure

| Path | Purpose |
| ---- | ------- |
| `src/` | React frontend source |
| `src-tauri/` | Rust and Tauri backend source |
| `locales/` | App localization files |
| `docs/developer/` | Architecture and development documentation |

## Development Commands

This project uses Vite+ command entrypoints.

```bash
vp install
vpr typecheck
vpr test:run
vpr rust:fmt:check
vpr rust:clippy
vpr tauri build --no-bundle
```

## GitHub Releases And Updates

GitHub Actions builds Windows and macOS release artifacts when a `v*` tag is pushed. The workflow publishes a GitHub Release with Windows MSI/NSIS installers, macOS DMG/app bundles, updater signatures, and `latest.json`.

To publish a release:

```bash
git tag v0.1.7
git push origin v0.1.7
```

The updater endpoint is configured as:

```text
https://github.com/DunHouGo/C4D-Plugin-Complier/releases/latest/download/latest.json
```

## License

This project is licensed under [GNU General Public License v2.0 only](LICENSE.md).
