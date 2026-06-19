# C4D Plugin Compiler User Guide

C4D Plugin Compiler builds and packages Cinema 4D C++ plugins with locally configured Maxon SDKs. It detects the compiler environment, resolves SDK roots or archives, runs the SDK build workflow, and creates release folders or zip files.

## Build Setup

- Plugin Root: the plugin module folder that contains `project/`, `source/`, and optional `res/`.
- Module: the SDK module folder. For 2026 CMake SDK builds, spaces are converted to a target-safe name internally.
- Package: the generated package folder name.
- C4D Versions: generated from buildable SDK sources only. Missing versions that only have an official download URL are shown in the SDK Matrix but skipped by default.
- Configuration: `Debug`, `Release`, or `Both`.
- Package Mode: `Merged`, `Per Version`, or `Both`.
- Artifact naming: output names use only the C4D major version; Release has no suffix and Debug adds `_Debug`.
- Output Dir: package output folder. Empty uses `Plugin Root\dist`.
- Zip, Clean, Refresh SDK: control archive output, output cleanup, and SDK cache refresh.

## SDK Sources

Use one SDK Root folder without spaces, such as `Documents\Maxon_SDK`. The app resolves SDKs in this order: configured SDK root, configured archive, installed Cinema 4D `sdk.zip`, cache root, then official download URL.

The SDK Matrix stays visible for known versions, but only SDK roots, archives, and installed `sdk.zip` files are treated as buildable by default. This prevents a missing 2025 install from silently entering a 2024.4 and 2026 local build.

## Build Logs

The build log is structured by timestamp, level, category, and message. Levels are color coded, category filters isolate SDK, CMake, Xcode, toolchain, package, or system messages, and Auto scroll can be disabled when inspecting earlier output.

Copy log and Save log export the currently filtered log view, including timestamp, level, and category.
