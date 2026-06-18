# C4D Plugin Compiler User Guide

C4D Plugin Compiler is a Rust and Tauri 2 desktop tool for building and packaging Cinema 4D C++ plugins. It manages C++ SDK sources for Cinema 4D 2024.4 and newer, checks CMake plus the Windows or macOS compiler environment, builds plugins through Maxon's official CMake preset workflow, and creates merged, per-version, and zip release artifacts.

## Main Interface

- Top left sidebar button: shows or hides the SDK Sources panel for SDK download and local path configuration.
- Top right sidebar button: shows or hides the Output Preview panel for generated file tree previews.
- Center workbench: edit plugin build parameters, inspect environment status, resolve SDKs, run builds, and review logs and artifacts.

## SDK Sources Parameters

- SDK Root: one shared SDK root folder. Use a path without spaces, such as `Documents\Maxon_SDK`. The tool creates version folders such as `2024_4`, `2025`, and `2026` under it.
- Auto Detect: detects locally installed Cinema 4D major versions, selects the smallest matching C++ SDK for each major version, such as `Cinema_4D_CPP_SDK_2026_0_0.zip` for 2026, and saves the default SDK root.
- Save: writes the current SDK root to `configs/sdk_sources.json`.
- Refresh: reloads the SDK root, local Cinema 4D installs, and available SDK list.
- SDK Matrix: available SDK versions. When local Cinema 4D installs are detected, the minimum supported version starts from the oldest installed local version. Extracted SDKs show as Ready; missing SDKs are downloaded from Maxon during resolve or build.
- Installed C4D: detected local Cinema 4D installs and the SDK version mapped to each major version.

SDK resolution order is: extracted SDKs under `SDK Root\<version>\sdk`, downloaded archives under `SDK Root\<version>\downloads`, then the official Maxon download URL. Installed Cinema 4D `sdk.zip` files are kept as a compatibility source. Automatic download URLs use Maxon's common pattern, such as `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip`, `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip`, and `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip`.

## Build Parameters

- Plugin Root: plugin source root, usually containing `project/`, `source/`, and optional `res/`. Supports directory picker and drag-and-drop.
- Module: C4D SDK module name and CMake target name, such as `postwatermark`.
- Package: release package name and output plugin folder name.
- C4D Versions: version tags generated from the SDK Sources start version.
- Configuration: build mode, one of `Debug`, `Release`, or `Both`.
- Package Mode: packaging mode, one of `Merged`, `Per Version`, or `Both`.
- Output Dir: artifact output folder. Empty uses `Plugin Root\dist`. Supports directory picker and drag-and-drop.
- Zip: create zip archives.
- Clean: remove old output folders before packaging.
- Refresh SDK: re-extract or re-download cached SDKs.
- Build: resolves SDKs, configures CMake, builds the module, and packages artifacts.
- Resolve SDKs: resolves SDK sources and refreshes the SDK Matrix without building.
- Refresh Environment: rechecks CMake, the platform compiler, the system SDK, and SDK configuration.
- Cancel: marks the current build job as cancelled.

## Output Preview

The right Output Preview panel derives a file tree from the current Package, C4D Versions, Configuration, Package Mode, Output Dir, and Zip settings. It does not write files; it previews the package folders, Windows `.xdl64` or macOS `.xlib` binaries, copied `res` location, and zip archives that will be generated.

## Notes

- This version supports Windows and macOS build workflows.
- Windows builds require CMake, Visual Studio 2022, and matching SDKs. macOS builds require CMake, Xcode 16, Clang, Python 3.8, and matching SDKs.
- Path fields can be typed manually, selected with the folder button, or filled by dropping a file or folder on the field.
- Cancel does not force-kill an already running CMake child process.
