# C4D Plugin Compiler 更新日志

## 未发布

- 修复原生 CMake 插件的包名与 `maxon_targetName` 不一致时构建不存在的 Visual Studio 工程的问题：构建命令现在读取 `CMakeLists.txt` 中的真实 target，例如将 `Boghma-WaterMark` 正确映射到 `Boghma_WaterMark`。
- 修复 Windows legacy SDK 的 `projecttool` 扫描嵌套插件外层目录时因缺少 Solution 定义而崩溃的问题：生成工程前会在 SDK 临时工作区为嵌套模块容器补充标准 `projectdefinition.txt`，macOS 构建流程保持不变。
- 修复 `Merged` 打包模式忽略包名的问题：合并包内的构建文件现在使用“包名 + 主版本号”格式，例如 `BackHighlight 2024.xlib`。
- 修复 Cinema 4D 2024.4 legacy SDK 编译嵌套模块时错误从 `plugins/<模块名>` 查找 Xcode 工程的问题：现在会保留完整的嵌套模块路径，并以实际模块名选择构建 scheme 和产物，支持 `BackHighlight/draw.back` 这类插件结构。
- 优化应用内下载速度：HTTP 下载客户端现在启用系统代理和 HTTP/2，并减少更新下载时的逐块日志写入，避免同一链接在应用内明显慢于浏览器。
- 修复一键配置或保存 SDK 根目录后编译工作台不会自动刷新 C4D 版本列表的问题：SDK 来源变更时会广播刷新事件，编译页和 SDK 配置页都会重新读取最新环境与版本状态，灰色版本会即时恢复可选。
- 为单个构建和队列构建补充耗时显示，并在任务完成后弹出提示；队列结束时继续显示汇总弹窗。
- 修复 Windows 下 `tauri dev` 绑定到被系统保留的开发端口导致无法启动的问题：开发服务器现在改用 `127.0.0.1:1680`，并同步调整 Tauri 的 `devUrl`。
- 修复构建工作台在切换 `Package Mode`、队列编辑和直接构建时可能读到旧 request 的问题：启动构建与加入队列现在都从当前 store 读取最新请求，避免界面选择和实际提交参数不一致。
- 修复打包时优先命中旧资源目录的问题：`copy_resources` 现在会优先使用最新构建产物旁边的 `res`，并忽略旧的 `dist-test-debug` 目录，避免把过期资源带进新包。
- 修复 `Merged` 打包模式被误改成按版本独立落盘的问题：Merged 现在恢复为单个包目录下按版本并排输出，而 `Per Version` 继续保持各自独立目录。
- 修复 2026 CMake SDK 打包偏离官方模块流程的问题：构建时会优先从 `sdk_custom_paths.txt` 检测真实模块名，打包时保留 Maxon 生成的模块二进制文件名，并让 Merged 与 Per Version 保持各自的目录语义，避免旧目录结构混入新包。
- 修复付费插件打包只复制 `res` 导致运行时依赖缺失的问题：输出目录现在会复制插件自带 `libs`，并保持官方插件格式下的任意额外库目录一起打包，避免遗漏插件自带资源。
- 修复偏好设置关于页面显示旧版本号的问题：版本号现在直接读取 Tauri 运行时版本，并在“检查更新”没有新版本时提示“当前已是最新版本”。
- 修复 legacy 构建后处理误把 `res/boghma.png` 之类二进制资源按 UTF-8 读取的问题，现在只扫描 `.vcxproj`、`.vcxproj.filters`、`SConscript`、`.cbp` 和 `project.pbxproj` 这些文本工程文件。
- 修复 Boghma WaterMark 插件工程里 `dist-test-debug` 先于 `res` 命中的旧 include 顺序，源码现在显式指向真实 `res/c4d_symbols.h` 和 `res/description/vpboghmawatermark.h`。
- 清理 legacy 生成工程中的 `dist-test-debug` 资源引用，降低重复打开 IDE 后再次命中旧副本的概率。
- 新增 `docs/c4d-plugin-compiler-principle.md`，说明工具原理、内部流程、问题根因和流程图。
- 在偏好设置中新增关于页面，展示当前应用版本，并提供手动检查更新、打开 GitHub 项目页和打开 GitHub Release 下载页的入口。

## 2026-06-22 v0.1.7

- 重新生成带密码的 Tauri updater signing key，并同步新的公开 `pubkey` 到 `tauri.conf.json`。
- 更新 GitHub Actions 的 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets。
- 将应用版本提升到 `0.1.7`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.6

- 移除 Release workflow 中空的 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 环境变量，避免无密码 updater key 被当作空密码 key 解码。
- 将应用版本提升到 `0.1.6`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.5

- 重新生成无密码 Tauri updater signing key，并同步新的公开 `pubkey` 到 `tauri.conf.json`。
- 更新 GitHub Actions 的 `TAURI_SIGNING_PRIVATE_KEY` secret，使发布构建可以生成签名 updater 产物。
- 将应用版本提升到 `0.1.5`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.4

- 修复 Release workflow 的 `tauriScript` 调用方式，改为 `npm run tauri`，确保 GitHub Actions 使用项目本地 Tauri CLI。
- 将应用版本提升到 `0.1.4`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.3

- 修复 Release workflow 的 `tauriScript` 配置，使用项目中的 `tauri` npm script，避免被 action 解析为不存在的 `npm run exec`。
- 将应用版本提升到 `0.1.3`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.2

- 修复 Release workflow 中不存在的 `tauri-apps/tauri-action@v1` 引用，改用官方存在的 `v0.6.2`。
- 调整 updater JSON 发布参数为当前 action 支持的 `includeUpdaterJson`。
- 将应用版本提升到 `0.1.2`，用于重新触发正式 GitHub Release 构建。

## 2026-06-22 v0.1.1

- 更新 README 展示图和模板来源说明，标注项目基于 `DunHouGo/tauri-desktop-starter` 扩展。
- 将应用版本提升到 `0.1.1`，用于触发新的 GitHub Release 构建。

## 2026-06-22

- 把 tag 发布流程改为构建成功后自动创建正式 GitHub Release，推送 `v*` tag 后可直接下载 Windows、macOS 和 updater 产物。
- 将 GitHub Actions 发布流程改为 Windows 和 macOS 双平台矩阵构建，修正 Tauri updater JSON 输入名并移除手动构建中的 frontend dist 产物上传。
- 恢复 Rust 后端编译调度模块 `src-tauri/src/compiler/build.rs`，修复 Windows GitHub Actions Tauri 构建时找不到 `compiler::build` 模块的问题。
- 使用 `npm@11.13.0` 重新同步 `package-lock.json` 的可选 peer 依赖，并将 GitHub Actions 中的 Vite+ 调用改为本地 `npm exec -- vp`，修复发布 workflow 的依赖安装失败和后续命令解析风险。
- 新增 GitHub Actions 手动测试构建 workflow，可在不创建 Release 的情况下验证 Windows 构建链路并上传构建产物。
- 同步 `package-lock.json` 中的 Tauri npm 包解析版本，修复 GitHub Actions `npm ci` 因锁文件不一致失败的问题。
- 将发布 workflow 收敛为仅由 `v*` tag 触发，发布版本时自动构建 Windows 安装包和 updater 产物，并恢复使用 `npm ci` 进行可复现安装。
- 将 Tauri 构建前命令改为使用本地 Vite+ CLI，避免 GitHub Actions 中缺少全局 `vpr`。
- 配置 GitHub Actions Windows 发布流程，生成安装包和 Tauri updater 的 `latest.json` 自动更新清单。
- 配置 updater GitHub Release 站点和签名公钥，并修正平台配置中的应用标题。
- 更新应用 README，移除模板说明，并将项目许可改为 GPL-2.0-only。
- 固定前端 Tauri npm 包版本，避免安装时升级到 2.10/2.11 后与 Rust crate 主次版本不匹配。

## 2026-06-20

- 拆分 Rust 后端大文件：`compiler/build`、`compiler/package` 和 `compiler/sdk` 现在按构建调度、CMake、legacy SDK、资源复制、命名、zip、SDK 配置、本机安装检测和版本规则拆成更小模块。
- 将 Rust 源代码中的注释统一改为中文，保留必要的技术名词和示例代码，便于本地维护。
- 清理 i18n 本地化：移除未使用的法语和阿语语言包，仅保留 `zh-CN` 和 `en-US`，并同步更新语言选择器和相关文档。
- 删除设置窗口中未使用的高级页面和模板示例翻译，设置页现在只保留 SDK 配置与外观两类实际功能。
- 按中文文案校准英文语言包，更新应用名称、设置说明、编译器副标题和队列空状态等文本，使英文界面与当前 C4D 插件编译器语义一致。
- 构建队列标题栏新增重新开始按钮，可把成功、失败或运行过的队列项重置为等待状态，便于一键重新运行整组多插件构建。
- 构建队列新增“保存为预设”和“加载预设”功能，队列预设会保存多个插件各自的多版本构建参数，方便后续一键恢复批量构建队列。
- 构建日志工具栏新增清空按钮，可以在保留筛选和自动滚动设置的同时清理当前日志列表。
- 修复编译工作台左右布局自适应问题，左侧队列工具栏不再被截断，右侧主面板会填满窗口剩余宽度。
- 修复跨版本 CMake SDK 构建时 Windows preset 固定为旧版 `windows_vs2022_v143` 的问题，2026 SDK 会自动使用 `windows_vs2022_v143_x64`，旧 SDK 仍回退到 `windows_vs2022_v143`。
- 修复 Windows 输出包只复制二进制、遗漏内嵌模块 `res` 的问题：每个输出插件文件夹都会包含 `res` 文件夹，可直接在 Cinema 4D 中识别为插件目录。
- 将队列预设从构建队列标题栏拆分为独立面板，支持新建、改名、加载、保存和删除，并继续持久化保存预设内容。
- 队列项现在可以编辑：点击编辑会把该项构建参数载入左侧表单，修改后可保存回原队列项，并会重置该项为等待状态。
- 构建队列新增上下移动按钮，可以在运行队列前调整多个插件构建任务的顺序。
- 修复 BackHighlight 这类外层包目录内嵌实际 SDK 模块目录的 2026 CMake 构建失败问题：当插件根目录内只有一个嵌套模块时，会自动使用该模块名作为 CMake target，例如 `draw.back`，不再错误构建外层包名 `BackHighlight`。

## 2026-06-19

- 新增构建队列模式：可将多个插件的构建参数加入队列，每个队列项保留自己的多 C4D 版本、构建配置和打包设置，并按顺序一次性编译。
- 队列面板新增加入队列、运行队列、清空队列和移除队列项操作；队列任务会显示插件名、版本标签和执行状态，失败时停止后续队列以便查看日志。
- 简化构建参数：移除界面上的 Module 输入，Package 会作为发布包名和内部 SDK 模块名；选择插件目录时会重新自动填充 Package，避免切换插件后沿用上一个包名。
- 将构建队列移动到左侧底部，并将日志和产物改为同一主面板内的标签页，避免日志与产物横向挤压。
- 修复 2024.4 legacy SDK 插件在生成嵌套或大小写不同的 Xcode 工程时构建失败的问题，例如 `Draw.back/draw.back/project/draw.back.xcodeproj`。
- 还原编译工作台中日志与产物并列的主内容布局，移除左侧 sidebar 和标题栏上的 sidebar 切换按钮；命令面板、菜单和快捷键中也同步移除 sidebar 开关入口。
- 重新做左侧 SDK 配置流程：新增“智能体检”和“一键配置”，按“检测本机 C4D -> 匹配 SDK -> 官方下载 zip -> 本地解压配置 -> 检查必要工具”的顺序展示状态。
- 新增 SDK 配置规则说明和必要环境清单；自动配置失败时，用户可以把已解压 SDK 或官方 SDK zip 放入 SDK 根目录后刷新重新扫描。
- 新增后端 SDK setup 报告命令，区分可自动准备的 SDK/CMake 提示和需要用户手动安装的 Xcode、Visual Studio、Windows SDK 等系统工具。
- 将 SDK 配置从左侧 sidebar 移到设置窗口的 SDK 配置页，避免窄侧栏截断路径、按钮和环境检查内容；左侧栏改为打开设置页的轻量入口。
- SDK 矩阵现在会标记未安装对应 Cinema 4D 的版本，并在选中详情中提示用户先安装对应版本的 C4D。
- 调整发布产物命名：版本号只保留 Cinema 4D 大版本，例如 `2024.4` 输出为 `2024`；Release 产物不再带配置后缀，只有 Debug 产物追加 `_Debug`。
- 修复未安装或未配置 2025 SDK 时仍默认触发 2025 构建的问题：构建版本现在只从本地已解析的 SDK root、SDK 压缩包或本机 `sdk.zip` 中自动生成，官方下载地址只作为可见来源，不再自动加入构建队列。
- 修复损坏 SDK 压缩包被误识别为可构建来源的问题：无效 zip 现在会显示为 `invalid configured archive` 或 `invalid installed sdk.zip`，不会自动加入构建队列；官方下载缓存损坏时会先删除再重新下载。
- 修复 2026 CMake SDK 构建带空格模块名时失败的问题，CMake target 会自动转换为无空格名称，并会清理旧模块别名链路，避免 `target_compile_definitions called with invalid arguments`。
- 修复保存构建日志失败的问题，日志文件现在通过 Rust 后端写入，不再受前端 `fs.write_text_file` scope 限制。
- 优化构建失败摘要，Xcode 或 Clang 失败时会优先显示真实 `error:` 附近上下文，不再只显示最后的构建命令摘要。
- 优化 SDK 解析速度，列出和解析 SDK 时不再对官方下载地址执行网络 HEAD 探测，避免缺失版本在配置阶段长时间卡住。
- 新增结构化构建日志系统：日志事件包含时间戳、等级和类别，日志面板支持颜色区分、等级筛选、类别筛选、自动滚动开关，并按当前筛选结果复制或另存为 `.log`。
- 修复点击构建按钮无响应的问题：SDK 解析、环境检测和 SDK 配置命令现在会在 Tauri blocking task 中执行，避免 `reqwest::blocking` 在 async runtime 中触发 Tokio runtime shutdown panic。
- 优化构建启动流程，SDK 解析失败时会立刻恢复失败状态并写入构建日志，不再继续启动构建任务或让界面停留在运行中状态。
- 修复 2024.4 legacy SDK 构建时复制插件目录失败的问题，工作区复制现在会跳过 `.git`、缓存、构建产物目录和 Unix socket 等特殊文件。
- 修复编译工作台右侧信息重复的 문제，产物和输出预览现在合并为同一面板的切换视图；同时修复构建日志区域无法滚动查看的问题，并在失败状态中直接显示最近错误。
- 修复 macOS 2024.4 legacy Xcode 工程 scheme 大小写不一致导致的构建失败，构建前会读取 Xcode scheme 列表并自动选择匹配的插件 scheme。
- 将产物和输出预览移动到右侧可折叠栏中，默认保持关闭；SDK 矩阵中已经解析到的配置项会显示为绿色状态文本。
- 构建失败摘要支持在进度面板内滚动查看；核心构建日志新增复制和另存为 `.log` 文件按钮，并允许直接选择日志文本复制。

## 2026-06-18

- 修复 SDK 根目录配置后的版本解析逻辑：SDK Matrix 现在始终保留 2024.4、2025 和 2026 可选项，并会在配置根目录中递归识别对应版本的 SDK 根目录或 SDK 压缩包。
- 调整 SDK 资源优先级为配置根目录、本机 Cinema 4D `sdk.zip`、本地缓存、官方下载，避免可用的 2024.4 本机 SDK 被 2026 或官方下载流程覆盖。
- 兼容旧版 `configs/sdk_sources.json` 分版本配置读取，避免已有 SDK 配置在新版本单根目录配置中被忽略。
- 将编译器内部构建缓存迁移到用户目录下的无空格路径，避免 Maxon CMake/Xcode 脚本在 `Application Support` 路径中生成 2026 工程失败。
- 将外部插件临时链接目录从 `modules` 改为 `plugin-links`，避免 Maxon CMake 将普通插件误判为需要 `exportedsymbols.txt` 的非插件模块。
- 清理构建时同步清理对应 SDK preset 的 CMake 生成目录，避免模块文件变化后继续复用过期工程缓存。
- 修复中文界面未覆盖主工作台的问题，编译参数、SDK Sources、Output Preview、状态面板、按钮和保存偏好提示现在会随语言设置切换。
- 新增 Plugin Root 自动识别插件名称，选择或拖入插件目录后会自动填充空的 Module 和 Package 字段。
- 调整主窗口默认布局，启动时默认隐藏左右侧栏，保留标题栏按钮用于按需展开。
- 新增 macOS C4D C++ 插件编译支持，使用 Maxon SDK `macos_universal_xcode` preset 生成 Xcode Universal 构建目录。
- 新增 macOS 环境检测，显示 Xcode、Clang、Python、CMake preset 和 `.xlib` 插件二进制扩展名。
- 新增 macOS Cinema 4D 安装中 `/Applications/Maxon Cinema 4D <version>/sdk.zip` 检测，并支持构建时创建模块符号链接。
- 更新输出预览和中英文用户指南，说明 Windows `.xdl64` 与 macOS `.xlib` 产物差异。

## 初始版本

- 基于 Tauri 2 starter 初始化 C4D Plugin Compiler 项目。
- 新增 Rust 后端模块：SDK 解析、环境检测、CMake 构建、打包和构建任务管理。
- 新增 Tauri commands：环境检测、SDK 解析、启动构建、取消构建、列出产物、打开产物目录。
- 新增编译工作台界面，包含构建参数、环境状态、SDK 矩阵、日志和产物列表。
- 新增 `configs/sdk_sources.json` 用于配置本地 SDK 或下载地址。
