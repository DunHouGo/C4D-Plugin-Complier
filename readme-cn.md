# C4D Plugin Compiler 用户指南

C4D Plugin Compiler 是一个基于 Rust 和 Tauri 2 的 Cinema 4D C++ 插件编译与打包工具。它会按目标 C4D 版本准备 SDK，检测 CMake 与 Visual Studio 环境，调用官方 CMake preset 编译插件，并生成合并包、分版本包和 zip 压缩包。

## 参数说明

- Plugin Root：插件源码根目录，目录内通常包含 `project/`、`source/` 和 `res/`。
- Module：C4D SDK 模块名，也是 CMake 构建 target 名，例如 `postwatermark`。
- Package：发布包名称，也是打包后的插件目录名称。
- Versions：目标 C4D 大版本，使用逗号分隔，例如 `2025, 2026`。
- Configuration：构建模式，可选 `Debug`、`Release`、`Both`。
- Package Mode：打包模式，可选 `Merged`、`Per Version`、`Both`。
- Output Dir：产物输出目录，留空时使用插件目录下的 `dist`。
- Zip：生成 zip 压缩包。
- Clean：打包前清理旧输出目录。
- Refresh SDK：忽略旧 SDK 缓存并重新解压或下载。

## SDK 来源

工具按以下顺序解析 SDK：

- `configs/sdk_sources.json` 中配置的 `sdk_root`、`sdk_zip` 或 `download_url`。
- 本机 `C:\Program Files\Maxon Cinema 4D <版本>\sdk.zip`。
- Maxon 官方 `downloads.json` 中的最新 C++ SDK 下载地址。

## 使用流程

- 点击刷新按钮检查 CMake、Visual Studio 2022 和 Windows SDK。
- 填写插件根目录、模块名、包名和版本。
- 点击 SDK 按钮预览各版本 SDK 来源。
- 点击 Build 开始构建。
- 构建完成后在 Artifacts 面板打开产物目录。

## 注意事项

- 第一版主要支持 Windows。
- 构建 C4D C++ 插件仍需要 CMake、Visual Studio 2022 和对应 C4D SDK。
- 取消按钮会标记任务取消，当前版本不会强制杀死已经运行的 CMake 子进程。
