# C4D Plugin Compiler User Guide

C4D Plugin Compiler is a Rust and Tauri 2 desktop tool for building and packaging Cinema 4D C++ plugins. It manages C++ SDK sources for Cinema 4D 2024.4 and newer, checks CMake plus the Windows or macOS compiler environment, builds plugins through Maxon's official CMake preset workflow, and creates merged, per-version, and zip release artifacts.

## Main Interface

- Both sidebars are hidden by default at startup. Use the title bar buttons to show or hide the SDK Sources panel and Output Preview panel when needed.
- Center workbench: edit plugin build parameters, inspect environment status, resolve SDKs, run builds, and review logs and artifacts.
- After choosing Simplified Chinese in Preferences, the main workbench, SDK Sources, Output Preview, buttons, statuses, and primary help text switch immediately.

## SDK Sources Parameters

- SDK Root: one shared SDK root folder. Use a path without spaces, such as `Documents\Maxon_SDK`. The tool creates version folders such as `2024_4`, `2025`, and `2026` under it.
- Auto Detect: detects locally installed Cinema 4D major versions, selects the smallest matching C++ SDK for each major version, such as `Cinema_4D_CPP_SDK_2026_0_0.zip` for 2026, and saves the default SDK root.
- Save: writes the current SDK root to `configs/sdk_sources.json`.
- Refresh: reloads the SDK root, local Cinema 4D installs, and available SDK list.
- SDK Matrix: available SDK versions. Extracted SDK roots, local SDK archives, and installed Cinema 4D `sdk.zip` files are treated as buildable sources and shown as ready. Versions with only an official download URL stay visible in the matrix, but they are not added to the build queue by default.
- Invalid SDK archives: if the SDK Matrix shows `invalid configured archive` or `invalid installed sdk.zip`, that zip is damaged or incomplete. It will not enter the build queue; delete or replace the archive, then refresh.
- Installed C4D: detected local Cinema 4D installs and the SDK version mapped to each major version.

SDK resolution order is: extracted SDKs under `SDK Root\<version>\sdk`, downloaded archives under `SDK Root\<version>\downloads`, then the official Maxon download URL. Installed Cinema 4D `sdk.zip` files are kept as a compatibility source. Automatic download URLs use Maxon's common pattern, such as `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip`, `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip`, and `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip`.

## Build Parameters

- Plugin Root: plugin source root, usually containing `project/`, `source/`, and optional `res/`. Supports directory picker and drag-and-drop. After selection, the last folder name is used to prefill empty `Module` and `Package` fields; manually entered values are kept.
- Module: C4D SDK module name, such as `postwatermark`. For 2026 CMake SDK builds, module names containing spaces are converted to a target-safe name internally, for example `Boghma WaterMark` builds as `Boghma_WaterMark`.
- Package: release package name and output plugin folder name.
- C4D Versions: version tags generated from the SDK Sources start version. Automatic selection only includes locally resolved SDK roots, SDK archives, or installed `sdk.zip` files; for example, if 2025 is not installed or configured, the build queue skips 2025.
- Configuration: build mode, one of `Debug`, `Release`, or `Both`.
- Package Mode: packaging mode, one of `Merged`, `Per Version`, or `Both`.
- Artifact naming: package names keep only the C4D major version, so `2024.4` outputs as `2024`; Release has no configuration suffix, while Debug adds `_Debug`.
- Output Dir: artifact output folder. Empty uses `Plugin Root\dist`. Supports directory picker and drag-and-drop.
- Zip: create zip archives.
- Clean: remove old output folders before packaging.
- Refresh SDK: re-extract or re-download cached SDKs.
- Build: resolves SDKs, configures CMake, builds the module, and packages artifacts.
- Resolve SDKs: resolves SDK sources and refreshes the SDK Matrix without building.
- Refresh Environment: rechecks CMake, the platform compiler, the system SDK, and SDK configuration.
- Cancel: marks the current build job as cancelled.

## Build Logs

- Each build log entry includes a timestamp, level, category, and message.
- Level colors: `info` is green, `warn` is amber, and `error` is red.
- Level filters: `All` shows everything, `Warn+` shows warnings plus errors, and `Errors` shows errors only.
- Category filters let you inspect `SDK`, `CMake`, `Xcode`, `Toolchain`, `Package`, or `System` messages.
- Auto scroll keeps the log pinned to the newest entry while it is enabled; turn it off to inspect older output.
- Copy log and save log export the currently filtered view, including timestamp, level, and category.

## Output Preview

The right Output Preview panel derives a file tree from the current Package, C4D Versions, Configuration, Package Mode, Output Dir, and Zip settings. It does not write files; it previews the package folders, Windows `.xdl64` or macOS `.xlib` binaries, copied `res` location, and zip archives that will be generated.

## Notes

- This version supports Windows and macOS build workflows.
- Windows builds require CMake, Visual Studio 2022, and matching SDKs. macOS builds require CMake, Xcode 16, Clang, Python 3.8, and matching SDKs.
- Path fields can be typed manually, selected with the folder button, or filled by dropping a file or folder on the field.
- If the selected Plugin Root is `.../MyPlugin` and `Module` plus `Package` are still empty, both fields are filled with `MyPlugin`.
- Build logs and backend errors keep their original English diagnostics so they can be searched against SDK, CMake, or compiler references.
- Cancel does not force-kill an already running CMake child process.
