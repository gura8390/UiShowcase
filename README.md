# 澄明界面展示（UiShowcase）

基于 **Rust + `egui`（`eframe` + OpenGL `glow` 后端 + `AccessKit` 无障碍桥接）** 构建的 Windows 桌面 UI 设计规范交互展示应用，配套完整工程设计规范文档 [设计规范草案_澄明原生工具风.md](./设计规范草案_澄明原生工具风.md)。

---

## 核心特性

- **高 DPI 锐利文字与双字体实时对比**
  - 通过 `build.rs` 向可执行文件直接嵌入 `PerMonitorV2` 清单（运行时 `GetProcessDpiAwareness == 2`）与多分辨率 `.ico` 应用图标；
  - 基于 OpenGL `glow` 按屏幕真实 `pixels_per_point` 1:1 光栅化字形图集，在 100% / 125% / 150% / 200% 缩放屏显下清晰锐利；
  - 内嵌 `FZGWKaiS.ttf`（方正公文楷体），顶栏与设置页支持一键实时切换「方正公文楷体 / 微软雅黑 UI」。
- **五大交互展示页面**
  1. **总览与色板**：等宽等高核心原则卡片、实时响应主题/强调色的语义色板网格、字体层级与等宽数据排版双栏对照；
  2. **典型工具界面**：统一工具栏、非阻塞模拟后台核查进度/取消（空闲态自动留白）、严格四列防重叠核查记录表、贴底对齐详情面板、空状态演示及危险操作中文模态二次确认；
  3. **控件与交互态**：五态按钮（默认/悬停/按下/禁用/键盘焦点）、内联校验错误、自绘现代下拉框、拨动开关、分段选择、自绘高对比度滑动条与等宽等高 2×2 状态反馈四件套；
  4. **主题与设置**：浅色 / 深色 / 跟随 Windows 系统（联动 DWM 深色标题栏）、4 组品牌强调色（澄明蓝 / 青瓷绿 / 暮山紫 / 琥珀金）、标准/紧凑双密度、关闭隐藏到托盘、真实 `HKCU\...\Run` 登录自启读写与 JSON 持久化；
  5. **关于与环境**：实时读取渲染后端、当前缩放倍率、`pixels_per_point`、视口逻辑/物理尺寸与系统托盘状态。
- **统一的交互与无障碍规范**
  - **文本可选统一原则**：所有按钮、导航项、分段切换、开关、下拉框、滑动条与可点击列表行禁止拖选复制；其余全部展示性文字均支持鼠标拖选与 `Ctrl+C` 复制，选区配色满足高对比度；
  - **全量 AccessKit -> Windows UI Automation 支持**：全部自绘控件显式注册中文无障碍名称、角色与状态，并配套 15 项自动化 UIA、几何防重叠与最小窗口（`820×540`）回归测试。
- **完整的 Windows 桌面工程闭环**
  - 自定义 `winit` 隐藏窗口事件循环包装器（`src/event_loop.rs`），窗口隐藏到托盘或最小化时零轮询空转且即时响应托盘「退出」「打开主窗口」与单实例唤醒；
  - Win32 原生托盘（`NOTIFYICON_VERSION_4` + `WS_EX_TOPMOST` + `WM_CONTEXTMENU` + KB135788），确保 Windows 11 右键托盘菜单始终位于任务栏前方；
  - 基于命名 `CreateMutexW` 的单实例排他与重复启动自动唤回已有窗口。

---

## 目录结构

```text
UiShowcase/
├── Cargo.toml                         # Rust 工程配置（eframe 0.33 / glow / accesskit / windows-sys）
├── build.rs                           # 编译期嵌入多分辨率应用图标与 PerMonitorV2 清单
├── assets/
│   ├── app.ico                        # 多分辨率应用图标
│   └── fonts/FZGWKaiS.ttf             # 内嵌方正公文楷体
├── src/
│   ├── main.rs                        # 单实例排他入口与窗口启动配置
│   ├── event_loop.rs                  # Windows 隐藏窗口事件循环包装器
│   ├── app.rs                         # 五页面主应用与状态管理
│   ├── theme.rs                       # 语义双主题色板、四组强调色、双密度与字体管理
│   ├── widgets.rs                     # 全套自绘现代控件、等高双栏/网格与 AccessKit 语义
│   ├── system.rs                      # Win32 原生托盘、DWM 标题栏、系统主题与开机自启
│   └── settings.rs                    # 本地 JSON 配置持久化与隔离环境支持
├── Tests/
│   └── Verify-UiAutomation.ps1        # 15 项 UI Automation、几何防重叠、最小窗口与托盘回归脚本
└── 设计规范草案_澄明原生工具风.md       # 配套 Rust + egui 桌面 UI 设计与工程规范文档
```

---

## 下载与运行

直接前往 [GitHub Releases](https://github.com/gura8390/UiShowcase/releases) 下载最新版本的单文件 `UiShowcase.exe` 双击即可运行（无需额外运行库或外部字体安装），同页亦提供对应版本的设计规范文档下载。

---

## 本地构建与自动化验收

环境要求：Windows 10/11 x64、Rust 1.95+（`x86_64-pc-windows-msvc`）与 MSVC C++ Build Tools。

```powershell
# 编译发布版
cargo build --release

# 运行 15 项 UI Automation、DPI、持久化、单实例、最小窗口与隐藏窗口托盘回归测试
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tests\Verify-UiAutomation.ps1
```

构建产物位于 `target\release\UiShowcase.exe`。
