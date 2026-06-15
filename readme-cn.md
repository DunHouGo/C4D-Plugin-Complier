# C4D Plugin Compiler 用户指南

C4D Plugin Compiler 是一个基于 Rust 和 Tauri 2 的 Cinema 4D C++ 插件编译与打包工具。它可以管理 Cinema 4D 2024.4 及之后版本的 C++ SDK 来源，检测 CMake、Visual Studio 2022 和 Windows SDK 环境，通过 Maxon 官方 CMake preset 构建插件，并生成合并包、分版本包和 zip 发布包。

## 主界面

- 顶部左侧侧栏按钮：显示或隐藏 SDK Sources 面板，用于配置 SDK 下载和本地路径。
- 顶部右侧侧栏按钮：显示或隐藏 Output Preview 面板，用于预览构建后将生成的文件树。
- 中间工作台：填写插件构建参数、查看环境状态、解析 SDK、执行构建、查看日志和产物。

## SDK Sources 参数

- SDK Root：统一的 SDK 根目录，建议使用无空格路径，例如 `Documents\Maxon_SDK`。工具会自动在其下创建 `2024_4`、`2025`、`2026` 等版本目录。
- Auto Detect：检测本机已安装的 Cinema 4D 大版本，自动选择该大版本内最小可用 C++ SDK，例如 2026 使用 `Cinema_4D_CPP_SDK_2026_0_0.zip`，并保存默认 SDK 根目录。
- Save：保存当前 SDK 根目录到 `configs/sdk_sources.json`。
- Refresh：重新读取 SDK 根目录、本机 Cinema 4D 安装和可用 SDK 列表。
- SDK Matrix：可用 SDK 版本列表。检测到本机 Cinema 4D 时，最低支持版本从本机最小安装版本开始；已解压的版本显示为 Ready，未下载的版本会在解析或构建时按 Maxon 下载地址自动下载。
- Installed C4D：本机 Cinema 4D 安装检测结果，并显示每个大版本对应的 SDK 版本。

SDK 解析顺序为：`SDK Root\<version>\sdk` 中已解压的 SDK、`SDK Root\<version>\downloads` 中已下载的 zip、Maxon 官方下载地址。本机安装目录中的 `sdk.zip` 作为兼容来源使用。自动下载地址采用 Maxon 常见格式，例如 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip`、`https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip` 和 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip`。

## 构建参数

- Plugin Root：插件源码根目录，通常包含 `project/`、`source/` 和可选的 `res/`。支持目录选择和拖拽。
- Module：C4D SDK 模块名，也是 CMake 构建 target 名，例如 `postwatermark`。
- Package：发布包名称，也是输出插件目录名称。
- C4D Versions：由 SDK Sources 中的起始版本自动生成的版本标签。
- Configuration：构建模式，可选 `Debug`、`Release` 或 `Both`。
- Package Mode：打包模式，可选 `Merged`、`Per Version` 或 `Both`。
- Output Dir：产物输出目录。留空时使用 `Plugin Root\dist`。支持目录选择和拖拽。
- Zip：生成 zip 压缩包。
- Clean：打包前清理旧输出目录。
- Refresh SDK：重新解压或下载 SDK 缓存。
- Build：解析 SDK、配置 CMake、构建模块并打包产物。
- Resolve SDKs：只解析 SDK 来源并刷新 SDK Matrix。
- Refresh Environment：重新检测 CMake、Visual Studio 2022、Windows SDK 和 SDK 配置。
- Cancel：取消当前构建任务标记。

## Output Preview

右侧 Output Preview 会根据当前 Package、C4D Versions、Configuration、Package Mode、Output Dir 和 Zip 设置生成文件树预览。它不会写入文件，只用于在构建前确认将生成的文件夹、`.xdl64` 二进制文件、`res` 资源复制位置和 zip 包结构。

## 注意事项

- 当前版本主要面向 Windows 构建流程。
- 构建 C4D C++ 插件仍需要本机安装 CMake、Visual Studio 2022 和对应 SDK。
- 路径输入既可以手动输入，也可以点击文件夹按钮选择，或将文件/目录拖拽到输入框区域。
- 取消任务不会强制杀死已经启动的 CMake 子进程。
