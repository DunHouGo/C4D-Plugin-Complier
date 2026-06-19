# C4D Plugin Compiler 用户指南

C4D Plugin Compiler 是一个基于 Rust 和 Tauri 2 的 Cinema 4D C++ 插件编译与打包工具。它可以管理 Cinema 4D 2024.4 及之后版本的 C++ SDK 来源，检测 CMake、Windows 或 macOS 编译环境，通过 Maxon 官方 CMake preset 构建插件，并生成合并包、分版本包和 zip 发布包。

## 主界面

- 启动时左右侧栏默认隐藏，标题栏两侧按钮可按需显示或隐藏 SDK Sources 面板与 Output Preview 面板。
- 中间工作台：填写插件构建参数、查看环境状态、解析 SDK、执行构建、查看日志和产物。
- 在偏好设置中选择简体中文后，主工作台、SDK Sources、Output Preview、按钮、状态和主要提示会立即切换为中文。

## SDK Sources 参数

- SDK Root：统一的 SDK 根目录，建议使用无空格路径，例如 `Documents\Maxon_SDK`。工具会自动在其下创建 `2024_4`、`2025`、`2026` 等版本目录。
- Auto Detect：检测本机已安装的 Cinema 4D 大版本，自动选择该大版本内最小可用 C++ SDK，例如 2026 使用 `Cinema_4D_CPP_SDK_2026_0_0.zip`，并保存默认 SDK 根目录。
- Save：保存当前 SDK 根目录到 `configs/sdk_sources.json`。
- Refresh：重新读取 SDK 根目录、本机 Cinema 4D 安装和可用 SDK 列表。
- SDK Matrix：可用 SDK 版本列表。已解压的 SDK root、本地 SDK 压缩包和本机 `sdk.zip` 会被视为可构建来源并显示为就绪状态；仅有官方下载地址的版本会保留在矩阵中用于提示，但不会默认加入构建队列。
- 无效 SDK 压缩包：如果 SDK Matrix 显示 `invalid configured archive` 或 `invalid installed sdk.zip`，表示该 zip 损坏或不是完整 zip。它不会进入构建队列，请删除或替换对应压缩包后刷新。
- Installed C4D：本机 Cinema 4D 安装检测结果，并显示每个大版本对应的 SDK 版本。

SDK 解析顺序为：`SDK Root\<version>\sdk` 中已解压的 SDK、`SDK Root\<version>\downloads` 中已下载的 zip、Maxon 官方下载地址。本机安装目录中的 `sdk.zip` 作为兼容来源使用。自动下载地址采用 Maxon 常见格式，例如 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip`、`https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip` 和 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip`。

## 构建参数

- Plugin Root：插件源码根目录，通常包含 `project/`、`source/` 和可选的 `res/`。支持目录选择和拖拽。选择后会自动根据路径最后一级目录名预填 `Module` 和 `Package`，如果这两个字段已经手动填写则不会覆盖。
- Module：C4D SDK 模块名，例如 `postwatermark`。2026 CMake SDK 构建时，包含空格的模块名会在内部转换为无空格 target，例如 `Boghma WaterMark` 会按 `Boghma_WaterMark` 构建。
- Package：发布包名称，也是输出插件目录名称。
- C4D Versions：由 SDK Sources 中的起始版本自动生成的版本标签。自动选择只包含本地已解析的 SDK root、SDK 压缩包或本机 `sdk.zip`，例如没有安装或配置 2025 时，构建队列会跳过 2025。
- Configuration：构建模式，可选 `Debug`、`Release` 或 `Both`。
- Package Mode：打包模式，可选 `Merged`、`Per Version` 或 `Both`。
- Output Dir：产物输出目录。留空时使用 `Plugin Root\dist`。支持目录选择和拖拽。
- Zip：生成 zip 压缩包。
- Clean：打包前清理旧输出目录。
- Refresh SDK：重新解压或下载 SDK 缓存。
- Build：解析 SDK、配置 CMake、构建模块并打包产物。
- Resolve SDKs：只解析 SDK 来源并刷新 SDK Matrix。
- Refresh Environment：重新检测 CMake、平台编译器、系统 SDK 和 SDK 配置。
- Cancel：取消当前构建任务标记。

## 构建日志

- 每条构建日志包含时间戳、等级、类别和消息正文。
- 等级颜色：`info` 使用绿色，`warn` 使用琥珀色，`error` 使用红色。
- 等级筛选：`全部` 显示所有日志，`警告+` 显示警告和错误，`错误` 只显示错误。
- 类别筛选：可按 `SDK`、`CMake`、`Xcode`、`工具链`、`打包` 和 `系统` 查看对应阶段日志。
- 自动滚动：开启时新日志会自动滚动到底部；关闭后可以停留在历史日志位置排查。
- 复制日志和另存日志会导出当前筛选后的日志内容，格式包含时间、等级和类别。

## Output Preview

右侧 Output Preview 会根据当前 Package、C4D Versions、Configuration、Package Mode、Output Dir 和 Zip 设置生成文件树预览。它不会写入文件，只用于在构建前确认将生成的文件夹、Windows 的 `.xdl64` 或 macOS 的 `.xlib` 二进制文件、`res` 资源复制位置和 zip 包结构。

## 注意事项

- 当前版本支持 Windows 和 macOS 构建流程。
- Windows 构建仍需要本机安装 CMake、Visual Studio 2022 和对应 SDK；macOS 构建需要 CMake、Xcode 16、Clang 和 Python 3.8。
- 路径输入既可以手动输入，也可以点击文件夹按钮选择，或将文件/目录拖拽到输入框区域。
- 如果拖入或选择的 Plugin Root 是 `.../MyPlugin`，并且 `Module` 和 `Package` 还是空的，它们会自动填成 `MyPlugin`。
- 构建日志和后端错误保留原始英文诊断信息，便于复制到 SDK 文档、CMake 或编译器错误搜索中排查。
- 取消任务不会强制杀死已经启动的 CMake 子进程。
