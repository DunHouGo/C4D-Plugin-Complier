# C4D Plugin Compiler 用户指南

C4D Plugin Compiler 是一个基于 Rust 和 Tauri 2 的 Cinema 4D C++ 插件编译与打包工具。它可以管理 Cinema 4D 2024.4 及之后版本的 C++ SDK 来源，检测 CMake、Windows 或 macOS 编译环境，通过 Maxon 官方 CMake preset 构建插件，并生成合并包、分版本包和 zip 发布包。

## 主界面

- 左侧工作区用于填写插件构建参数，底部显示构建队列。
- 中间工作台用于查看环境状态、解析 SDK、执行构建，并通过“构建日志 / 产物”标签页切换查看输出。
- 偏好设置包含 SDK 配置、外观和关于页面。在外观中选择简体中文后，主工作台、SDK Sources、Output Preview、按钮、状态和主要提示会立即切换为中文。

## SDK Sources 参数

- SDK Root：统一的 SDK 根目录，建议使用无空格路径，例如 `Documents\Maxon_SDK`。工具会自动在其下创建 `2024_4`、`2025`、`2026` 等版本目录。
- Smart Check：检测本机已安装的 Cinema 4D 大版本，自动选择该大版本内最小可用 C++ SDK，例如 2026 使用 `Cinema_4D_CPP_SDK_2026_0_0.zip`，并报告缺失的 SDK 与工具链。
- One-click Setup：创建 SDK 根目录，下载所需的 Maxon 官方 SDK zip，先解压到临时目录校验，再写入对应版本缓存；下载或解压失败会显示在检查报告中，不会留下半截缓存。
- Save：保存当前 SDK 根目录到用户配置目录中的 `sdk_sources.json`。未保存过配置时会使用当前系统的文档目录；仓库内旧的 `configs/sdk_sources.json` 仅在旧检出中存在时作为兼容读取，不会在运行时写入。
- Refresh：重新读取 SDK 根目录、本机 Cinema 4D 安装和可用 SDK 列表。
- SDK Matrix：可用 SDK 版本列表。已解压的 SDK root 和本地 SDK 压缩包会被视为可构建来源并显示为就绪状态；仅有官方下载地址的版本会保留在矩阵中用于一键配置和构建时下载。本机安装目录中的 `sdk.zip` 只作为没有官方扩展 SDK URL 时的兼容兜底来源。完成一键配置或保存 SDK 根目录后，工作台会自动重新刷新版本列表，灰色版本通常会立即恢复为可选。
- 无效 SDK 压缩包：如果 SDK Matrix 显示 `invalid configured archive` 或 `invalid installed sdk.zip`，表示该 zip 损坏或不是完整 zip。它不会进入构建队列，请删除或替换对应压缩包后刷新。
- Installed C4D：本机 Cinema 4D 安装检测结果，并显示每个大版本对应的 SDK 版本。

SDK 解析顺序为：`SDK Root\<version>\sdk` 中已解压的 SDK、`SDK Root\<version>\downloads` 中已下载的 zip、Maxon 官方下载地址，最后才回退到本机安装目录中的 `sdk.zip`。自动下载地址采用 Maxon 常见格式，例如 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip`、`https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip` 和 `https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip`。

## 构建参数

- Plugin Root：插件源码根目录，通常包含 `project/`、`source/` 和可选的 `res/`。支持目录选择和拖拽。选择后会自动根据路径最后一级目录名预填 `Package`。
- Package：发布包名称和输出插件目录名称。SDK 模块名会保持和官方插件结构一致：`sdk_custom_paths.txt` 可以提供 `postwatermark` 这类模块别名，直接插件根目录使用其模块文件夹名，单个嵌套 SDK 模块例如 `BackHighlight/draw.back/project/projectdefinition.txt` 会作为实际构建 target。2024.4 legacy SDK 会保留完整的嵌套路径来生成和定位 Xcode 工程。打包后的二进制文件保留官方模块文件名，例如 `postwatermark.xdl64`。
- C4D Versions：由 SDK Sources 中的起始版本自动生成的版本标签。自动选择只包含本地已解析的 SDK root 或 SDK 压缩包，例如没有安装或配置 2025 时，构建队列会跳过 2025。
- Configuration：构建模式，可选 `Debug`、`Release` 或 `Both`。
- Package Mode：打包模式，可选 `Merged`、`Per Version` 或 `Both`。Merged 会保留一个总输出目录，并把每个版本/配置的二进制直接放入其中；Per Version 则为每个版本生成独立顶层目录。
- 产物命名：发布包文件夹只保留 C4D 大版本号，例如 `2024.4` 会输出为 `2024`；Release 不加配置后缀，Debug 会追加 `_Debug`。二进制文件名保留 Maxon 构建系统生成的官方 SDK 模块名。
- Output Dir：产物输出目录。留空时使用 `Plugin Root\dist`。支持目录选择和拖拽。
- Zip：生成 zip 压缩包。
- Clean：打包前清理旧输出目录。
- Refresh SDK：重新解压或下载 SDK 缓存。
- Build：解析 SDK、配置 CMake、构建模块并打包产物。
- Add to Queue：把当前 `Plugin Root`、`Package`、`C4D Versions`、构建配置、打包模式和输出设置保存为一个队列项。随后可以切换到另一个插件目录，再加入新的队列项。编辑队列项时，此按钮会变为更新队列项。
- Run Queue：按队列顺序逐个编译插件。每个队列项仍会编译它自己的多个 C4D 版本，因此可以一次性完成“多个插件 × 多个版本”的发布构建。
- Clear Queue：清空尚未运行或已完成的队列记录。
- Resolve SDKs：只解析 SDK 来源并刷新 SDK Matrix。
- Refresh Environment：重新检测 CMake、平台编译器、系统 SDK 和 SDK 配置。
- Cancel：请求停止当前构建任务标记。已经启动的 CMake 子进程不会被强制杀死；队列模式下，当前构建结束后不会继续后续队列项。

## 队列模式

- 队列项会复制加入队列时的完整构建参数，后续修改左侧表单不会影响已经加入的队列。
- 每个队列项显示插件名、版本标签、构建配置、打包模式和当前状态。
- 点击队列项的编辑按钮会把该项参数载入左侧表单，修改后点击更新队列项保存；上下箭头可以调整队列顺序。
- 队列按顺序串行执行，避免多个 CMake/SDK 准备流程同时写入同一缓存目录。
- 如果某个队列项构建失败，队列会停止，方便先查看日志并修复对应插件。
- 构建日志会连续记录整个队列流程，并在每个队列项开始时写入插件名和版本列表。

## 构建日志

- 每条构建日志包含时间戳、等级、类别和消息正文。
- 等级颜色：`info` 使用绿色，`warn` 使用琥珀色，`error` 使用红色。
- 等级筛选：`全部` 显示所有日志，`警告+` 显示警告和错误，`错误` 只显示错误。
- 类别筛选：可按 `SDK`、`CMake`、`Xcode`、`工具链`、`打包` 和 `系统` 查看对应阶段日志。
- 自动滚动：开启时新日志会自动滚动到底部；关闭后可以停留在历史日志位置排查。
- 复制日志和另存日志会导出当前筛选后的日志内容，格式包含时间、等级和类别。

## 产物

产物标签页会显示当前构建生成的包目录和 zip 文件。点击 Open 可以在系统文件管理器中打开对应产物位置。

## 关于与更新

- 关于页面会显示当前应用版本号，用于判断自动更新是否需要安装新版。
- Check for Updates 会手动检查 GitHub Release updater 清单；发现新版后可以直接下载、安装并重启应用。
- Open Downloads 会打开 GitHub Release 下载页面，用户可以手动下载安装包。
- Open GitHub 会打开项目 GitHub 页面，便于查看源码、发布记录和问题反馈。

## 注意事项

- 当前版本支持 Windows 和 macOS 构建流程。
- Windows 构建仍需要本机安装 CMake、Visual Studio 2022 和对应 SDK；macOS 构建需要 CMake、Xcode 16、Clang 和 Python 3.8。
- Windows legacy SDK 构建会直接调用 Maxon `projecttool` 和目标 `.vcxproj`，不需要手动以管理员身份运行 `generate_solution_win.bat`。
- 路径输入既可以手动输入，也可以点击文件夹按钮选择，或将文件/目录拖拽到输入框区域。
- 如果拖入或选择的 Plugin Root 是 `.../MyPlugin`，Package 会自动填成 `MyPlugin`。
- 构建日志和后端错误保留原始英文诊断信息，便于复制到 SDK 文档、CMake 或编译器错误搜索中排查。
- 取消任务不会强制杀死已经启动的 CMake 子进程。
