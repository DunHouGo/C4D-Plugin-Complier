# C4D Plugin Compiler User Guide

C4D Plugin Compiler builds and packages Cinema 4D C++ plugins with locally configured Maxon SDKs. It detects the compiler environment, resolves SDK roots or archives, runs the SDK build workflow, and creates release folders or zip files.

## Build Setup

- Plugin Root: the plugin module folder that contains `project/`, `source/`, and optional `res/`.
- Package: the generated package folder name and internal SDK module name. Selecting Plugin Root fills it from the folder name; editing Package updates the internal module name too. For 2026 CMake SDK builds, spaces are converted to a target-safe name internally, and a single nested SDK module such as `BackHighlight/draw.back/project/projectdefinition.txt` is used as the actual target.
- C4D Versions: generated from buildable SDK sources only. Missing versions that only have an official download URL are shown in the SDK Matrix but skipped by default.
- Configuration: `Debug`, `Release`, or `Both`.
- Package Mode: `Merged`, `Per Version`, or `Both`.
- Artifact naming: output names use only the C4D major version; Release has no suffix and Debug adds `_Debug`.
- Output Dir: package output folder. Empty uses `Plugin Root\dist`.
- Zip, Clean, Refresh SDK: control archive output, output cleanup, and SDK cache refresh.

## SDK Sources

Use one SDK Root folder without spaces, such as `Documents\Maxon_SDK`. The app resolves SDKs in this order: configured SDK root, configured archive, cache root, official download URL, then installed Cinema 4D `sdk.zip` only as a compatibility fallback.

SDK root preferences are saved in the user config directory. When no preference exists, the app uses the current system Documents folder. A legacy repository `configs/sdk_sources.json` file is read only when it exists from an older checkout.

Smart Check reports detected Cinema 4D installs, SDK availability, and required build tools without changing files. One-click Setup creates the SDK root, downloads the required official Maxon SDK zip, extracts it into a temporary directory for validation, then writes the version cache.

The SDK Matrix stays visible for known versions, but only SDK roots and archives are treated as buildable by default. This prevents a missing 2025 SDK from silently entering a 2024.4 and 2026 local build.

## Queue Mode

Add to Queue copies the current build settings into a queue item. Edit loads a queue item back into the left form, Update Queue Item saves the changed settings, and the arrow buttons reorder queued builds before Run Queue executes them serially.

## Build Logs

The build log is structured by timestamp, level, category, and message. Levels are color coded, category filters isolate SDK, CMake, Xcode, toolchain, package, or system messages, and Auto scroll can be disabled when inspecting earlier output.

Copy log and Save log export the currently filtered log view, including timestamp, level, and category.
