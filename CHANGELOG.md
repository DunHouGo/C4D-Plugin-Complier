# 更新日志

## 2026-06-19

- 修复未安装或未配置 2025 SDK 时仍默认触发 2025 构建的问题：构建版本现在只从本地已解析的 SDK root、SDK 压缩包或本机 `sdk.zip` 中自动生成，官方下载地址只作为可见来源，不再自动加入构建队列。
- 修复损坏 SDK 压缩包被误识别为可构建来源的问题：无效 zip 现在会显示为 `invalid configured archive` 或 `invalid installed sdk.zip`，不会自动加入构建队列；官方下载缓存损坏时会先删除再重新下载。
- 修复 2026 CMake SDK 构建带空格模块名时失败的问题，CMake target 会自动转换为无空格名称，并会清理旧模块别名链接，避免 `target_compile_definitions called with invalid arguments`。
- 修复保存构建日志失败的问题，日志文件现在通过 Rust 后端写入，不再受前端 `fs.write_text_file` scope 限制。
- 优化构建失败摘要，Xcode 或 Clang 失败时会优先显示真实 `error:` 附近上下文，不再只显示最后的构建命令摘要。
- 优化 SDK 解析速度，列出和解析 SDK 时不再对官方下载地址执行网络 HEAD 探测，避免缺失版本在配置阶段长时间卡住。
- 新增结构化构建日志系统：日志事件包含时间戳、等级和类别，日志面板支持颜色区分、等级筛选、类别筛选、自动滚动开关，并按当前筛选结果复制或另存为 `.log`。
- 修复点击构建按钮无响应的问题：SDK 解析、环境检测和 SDK 配置命令现在会在 Tauri blocking task 中执行，避免 `reqwest::blocking` 在 async runtime 中触发 Tokio runtime shutdown panic。
- 优化构建启动流程，SDK 解析失败时会立即恢复失败状态并写入构建日志，不再继续启动构建任务或让界面停留在运行中状态。
- 修复 2024.4 legacy SDK 构建时复制插件目录失败的问题，工作区复制现在会跳过 `.git`、缓存、构建产物目录和 Unix socket 等特殊文件。
- 修复编译工作台右侧信息重复的问题，产物和输出预览现在合并为同一面板的切换视图；同时修复构建日志区域无法滚动查看的问题，并在失败状态中直接显示最近错误。
- 修复 macOS 2024.4 legacy Xcode 工程 scheme 大小写不一致导致的构建失败，构建前会读取 Xcode scheme 列表并自动选择匹配的插件 scheme。
- 将产物和输出预览移动到右侧可折叠栏中，默认保持关闭；SDK 矩阵中已经解析到的配置项会显示为绿色状态文字。
- 构建失败摘要支持在进度面板内滚动查看；核心构建日志新增复制和另存为 `.log` 文件按钮，并允许直接选择日志文本复制。

## 2026-06-18

- 修复 SDK 根目录配置后的版本解析逻辑：SDK Matrix 现在始终保留 2024.4、2025 和 2026 可选项，并会在配置根目录中递归识别对应版本的 SDK 根目录或 SDK 压缩包。
- 调整 SDK 来源优先级为配置根目录、本机 Cinema 4D `sdk.zip`、本地缓存、官方下载，避免可用的 2024.4 本机 SDK 被 2026 或官方下载流程覆盖。
- 兼容旧版 `configs/sdk_sources.json` 分版本配置读取，避免已有 SDK 配置在新版单根目录配置中被忽略。
- 将编译器内部构建缓存迁移到用户目录下的无空格路径，避免 Maxon CMake/Xcode 脚本在 `Application Support` 路径中生成 2026 工程失败。
- 将外部插件临时链接目录从 `modules` 改为 `plugin-links`，避免 Maxon CMake 将普通插件误判为需要 `exportedsymbols.txt` 的非插件模块。
- 清理构建时同步清理对应 SDK preset 的 CMake 生成目录，避免模块文件变化后继续复用过期工程缓存。
- 修复中文界面未覆盖主工作台的问题，编译参数、SDK Sources、Output Preview、状态面板、按钮和保存偏好提示现在会随语言设置切换。
- 新增 Plugin Root 自动识别插件名称，选择或拖入插件目录后会自动填充空的 Module 和 Package 字段。
- 调整主窗口默认布局，启动时默认隐藏左右侧栏，保留标题栏按钮用于按需展开。
- 新增 macOS C4D C++ 插件编译支持，使用 Maxon SDK `macos_universal_xcode` preset 生成 Xcode Universal 构建目录。
- 新增 macOS 环境检测，显示 Xcode、Clang、Python、CMake preset 和 `.xlib` 插件二进制扩展名。
- 新增 macOS Cinema 4D 安装与 `/Applications/Maxon Cinema 4D <version>/sdk.zip` 检测，并支持构建时创建模块符号链接。
- 更新输出预览和中英文用户指南，说明 Windows `.xdl64` 与 macOS `.xlib` 产物差异。

## 2026-06-15

- 移除左侧 SDK Sources 中重复的 C4D 起始版本选择，并让 SDK Matrix 与构建版本下限自动从本机最小 Cinema 4D 安装版本开始。
- 简化 SDK Sources 配置为单一 SDK 根目录，新增自动检测本机 Cinema 4D 版本、自动映射 Maxon C++ SDK 下载地址和按版本目录缓存 SDK 的流程。
- 修复左侧 SDK Sources 面板版本列表为空的问题，SDK 版本列表不再依赖联网探测，并兼容 Cinema 4D 2024.4 对应的本机 2024 安装目录。
- 修复标题栏左右侧栏按钮不生效的问题，恢复可折叠的左侧 SDK 配置栏和右侧文件树预览栏。
- 新增 SDK Sources 面板，支持 Cinema 4D 2024.4 及之后版本的 SDK root、sdk.zip 和 download URL 配置。
- 新增 C4D 起始版本选择，默认从 2024.4 开始，并自动选中该版本之后的可用 SDK 版本作为构建标签。
- 将 Plugin Root、Output Dir 和 SDK 路径配置升级为支持文件/目录选择和拖拽的路径输入。
- 新增输出文件树预览，根据当前包名、版本、构建配置、打包模式和 zip 开关展示将生成的目录结构。
- 为构建参数、SDK 配置和主要按钮增加问号提示图标，鼠标悬浮可查看说明。
- 新增 Rust SDK 配置命令，用于读取、保存、删除 SDK 来源，并列出可用 SDK 版本。
- 更新中英文用户指南和开发文档，说明侧栏、路径选择、版本选择和 SDK 配置流程。

## 初始版本

- 基于 Tauri 2 starter 初始化 C4D Plugin Compiler 项目。
- 新增 Rust 后端模块：SDK 解析、环境检测、CMake 构建、打包和构建任务管理。
- 新增 Tauri commands：环境检测、SDK 解析、启动构建、取消构建、列出产物、打开产物目录。
- 新增编译工作台界面，包含构建参数、环境状态、SDK 矩阵、日志和产物列表。
- 新增 `configs/sdk_sources.json` 用于配置本地 SDK 或下载地址。
