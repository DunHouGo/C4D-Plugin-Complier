# 更新日志

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
