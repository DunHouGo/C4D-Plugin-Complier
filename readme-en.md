# C4D Plugin Compiler User Guide

C4D Plugin Compiler is a Rust and Tauri 2 desktop tool for building and packaging Cinema 4D C++ plugins. It manages C++ SDK sources for Cinema 4D 2024.4 and newer, checks CMake plus the Windows or macOS compiler environment, builds plugins through Maxon's official CMake preset workflow, and creates merged, per-version, and zip release artifacts.

## Main Interface

- The left work area edits plugin build parameters and shows the build queue at the bottom.
- The center workbench inspects environment status, resolves SDKs, runs builds, and switches between Build Log and Artifacts tabs.
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

- Plugin Root: plugin source root, usually containing `project/`, `source/`, and optional `res/`. Supports directory picker and drag-and-drop. After selection, the last folder name is used to prefill `Package`.
- Package: release package name, internal SDK module name, and output plugin folder name. Selecting Plugin Root fills it from the folder name; editing Package updates the internal module name too. For 2026 CMake SDK builds, names containing spaces are converted to a target-safe name internally, for example `Boghma WaterMark` builds as `Boghma_WaterMark`; when the plugin root contains one nested SDK module, such as `BackHighlight/draw.back/project/projectdefinition.txt`, the nested module name is used as the actual CMake target.
- C4D Versions: version tags generated from the SDK Sources start version. Automatic selection only includes locally resolved SDK roots, SDK archives, or installed `sdk.zip` files; for example, if 2025 is not installed or configured, the build queue skips 2025.
- Configuration: build mode, one of `Debug`, `Release`, or `Both`.
- Package Mode: packaging mode, one of `Merged`, `Per Version`, or `Both`.
- Artifact naming: package names keep only the C4D major version, so `2024.4` outputs as `2024`; Release has no configuration suffix, while Debug adds `_Debug`.
- Output Dir: artifact output folder. Empty uses `Plugin Root\dist`. Supports directory picker and drag-and-drop.
- Zip: create zip archives.
- Clean: remove old output folders before packaging.
- Refresh SDK: re-extract or re-download cached SDKs.
- Build: resolves SDKs, configures CMake, builds the module, and packages artifacts.
- Add to Queue: saves the current `Plugin Root`, `Package`, `C4D Versions`, build configuration, package mode, and output settings as one queue item. You can then switch to another plugin folder and add another item. While editing a queue item, this button changes to update that item.
- Run Queue: builds queued plugins one by one. Each queue item still builds its own selected C4D versions, so one run can cover multiple plugins across multiple versions.
- Clear Queue: removes queued and completed records.
- Resolve SDKs: resolves SDK sources and refreshes the SDK Matrix without building.
- Refresh Environment: rechecks CMake, the platform compiler, the system SDK, and SDK configuration.
- Cancel: requests cancellation for the current build job. Already-started CMake child processes are not force-killed; in queue mode, the queue will stop after the current build finishes.

## Queue Mode

- A queue item copies the full build settings at the moment it is added, so later form edits do not change existing queue items.
- Each item shows the plugin name, version tags, build configuration, package mode, and current status.
- Use a queue item's edit button to load its settings back into the left form, then update the item after making changes. The up and down arrow buttons reorder queued builds.
- The queue runs serially to avoid multiple CMake or SDK preparation steps writing into the same cache directories at once.
- If a queue item fails, the queue stops so you can inspect the log and fix that plugin first.
- Build logs are continuous for the whole queue and include the plugin name plus version list when each item starts.

## Build Logs

- Each build log entry includes a timestamp, level, category, and message.
- Level colors: `info` is green, `warn` is amber, and `error` is red.
- Level filters: `All` shows everything, `Warn+` shows warnings plus errors, and `Errors` shows errors only.
- Category filters let you inspect `SDK`, `CMake`, `Xcode`, `Toolchain`, `Package`, or `System` messages.
- Auto scroll keeps the log pinned to the newest entry while it is enabled; turn it off to inspect older output.
- Copy log and save log export the currently filtered view, including timestamp, level, and category.

## Artifacts

The Artifacts tab shows package folders and zip files generated by the current build. Use Open to reveal each artifact in the system file manager.

## Notes

- This version supports Windows and macOS build workflows.
- Windows builds require CMake, Visual Studio 2022, and matching SDKs. macOS builds require CMake, Xcode 16, Clang, Python 3.8, and matching SDKs.
- Path fields can be typed manually, selected with the folder button, or filled by dropping a file or folder on the field.
- If the selected Plugin Root is `.../MyPlugin`, Package is filled with `MyPlugin`.
- Build logs and backend errors keep their original English diagnostics so they can be searched against SDK, CMake, or compiler references.
- Cancel does not force-kill an already running CMake child process.
