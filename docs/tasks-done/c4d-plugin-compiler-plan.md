# C4D Plugin Compiler 已完成计划

## 已完成

- 基于 `DunHouGo/tauri-desktop-starter` 初始化项目。
- 使用 Rust/Tauri 2 实现环境检测、SDK 解析、CMake 构建、任务事件和打包模块。
- 前端首页替换为 C4D 插件编译工作台，包含配置区、环境状态、SDK 矩阵、日志和产物列表。
- 添加 `configs/sdk_sources.json` 作为 SDK 来源覆盖配置。
- 添加中文和英文用户指南。
- 更新应用名称、Tauri 标识、窗口尺寸和发布描述。

## 待完善

- 强制终止已启动的 CMake 子进程。
- macOS 和 Linux 构建流程。
- 更细粒度的下载进度事件。
- 用真实插件执行完整 `2025-2026 Release Both zip` 验收。
