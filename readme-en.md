# C4D Plugin Compiler User Guide

C4D Plugin Compiler is a Rust and Tauri 2 desktop tool for building and packaging Cinema 4D C++ plugins. It prepares SDKs for selected Cinema 4D versions, checks the CMake and Visual Studio environment, builds through the official CMake preset workflow, and creates merged, per-version, and zip release artifacts.

## Parameters

- Plugin Root: plugin source root, usually containing `project/`, `source/`, and `res/`.
- Module: C4D SDK module name and CMake target name, such as `postwatermark`.
- Package: release package name and output plugin folder name.
- Versions: target C4D major versions separated by commas, such as `2025, 2026`.
- Configuration: build mode, one of `Debug`, `Release`, or `Both`.
- Package Mode: packaging mode, one of `Merged`, `Per Version`, or `Both`.
- Output Dir: artifact output folder; defaults to `dist` under the plugin root.
- Zip: create zip archives.
- Clean: remove old output folders before packaging.
- Refresh SDK: ignore existing SDK cache and re-extract or re-download.

## SDK Sources

The tool resolves SDKs in this order:

- `sdk_root`, `sdk_zip`, or `download_url` from `configs/sdk_sources.json`.
- Installed `C:\Program Files\Maxon Cinema 4D <version>\sdk.zip`.
- The latest official C++ SDK URL from Maxon's `downloads.json`.

## Workflow

- Refresh the environment report to check CMake, Visual Studio 2022, and Windows SDK.
- Fill in the plugin root, module name, package name, and versions.
- Resolve SDKs to preview the source for each version.
- Start the build.
- Open generated artifacts from the Artifacts panel.

## Notes

- The first version focuses on Windows.
- Building C4D C++ plugins still requires CMake, Visual Studio 2022, and the matching C4D SDK.
- Cancel marks a job as cancelled; the current version does not force-kill an already running CMake child process.
