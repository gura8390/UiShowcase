# 澄明界面展示（UiShowcase）

基于 **Rust + `egui`（`eframe` + OpenGL `glow` 后端 + `AccessKit` 无障碍桥接）** 构建的 Windows 桌面 UI 设计规范交互展示应用，配套完整工程设计规范文档 [设计规范草案_澄明原生工具风.md](./设计规范草案_澄明原生工具风.md)（同步作为工作区规范 `D:\Codex\_specs\egui-clear-ui.md`）。

---

## 核心特性

- **1:9 黄金分栏与纯净窗口骨架**
  - 左侧导航分栏仅承载 3 个内容页面切换，随窗口缩放严格保持 `1:9` 比例（`76..=280 pt`）且支持自由拖动微调；宽度 `< 148 pt` 时自动切换居中紧凑简称（保留完整 AccessKit 名称与悬停提示）；
  - 右上角固定放置 **「设置」** 与 **「关于」** 入口，外观/字体/主题/强调色/密度统一收归「设置」弹窗管理；
  - 窗口本体仅保留右上角原生关闭键这唯一退出入口，正文下方无低信息含量状态栏。
- **两类统一动效标准（尊重系统「减少动画」且零空闲开销）**
  - **切换式动画**：左侧 3px 品牌强调色指示条 `140 ms` 平滑跟滑 + 内容区方向性纵向微滑入（`±14 pt`）与 `150 ms` 淡入；
  - **弹出式动画**：「设置」、「关于」与「二次确认」全部采用统一居中模态弹窗，从主窗口正中央 `0.88 → 1.0` 缓出缩放 + `160 ms` 淡入。
- **高 DPI 锐利文字与双字体实时切换**
  - 通过 `build.rs` 向可执行文件直接嵌入 `PerMonitorV2` 清单（运行时 `GetProcessDpiAwareness == 2`）与多分辨率 `.ico` 应用图标；
  - 基于 OpenGL `glow` 按屏幕真实 `pixels_per_point` 1:1 光栅化字形图集，在 100% / 125% / 150% / 200% 缩放屏显下清晰锐利；
  - 内嵌 `FZGWKaiS.ttf`（方正公文楷体），在「设置」弹窗内支持一键切换「方正公文楷体 / 微软雅黑 UI」。
- **三大内容页 + 两大居中弹窗**
  1. **总览与色板**：等宽等高核心原则卡片、实时响应主题/强调色的语义色板网格、字体层级与等宽数据排版双栏对照；
  2. **典型工具界面**：统一工具栏、非阻塞模拟后台核查进度/取消（空闲态自动留白）、严格四列防重叠核查记录表、贴底对齐详情面板、空状态演示及危险操作居中模态二次确认；
  3. **控件与交互态**：五态按钮（默认/悬停/按下/禁用/键盘焦点）、内联校验错误、自绘现代下拉框、拨动开关、分段选择、自绘高对比度滑动条与等宽等高 2×2 状态反馈四件套；
  4. **主题与设置（居中弹窗）**：浅色 / 深色 / 跟随 Windows 系统（联动 DWM 深色标题栏）、4 组品牌强调色（澄明蓝 / 青瓷绿 / 暮山紫 / 琥珀金）、标准/紧凑双密度、关闭隐藏到托盘、基于 Windows 启动文件夹 `.lnk` 快捷方式的开机自启管理（不写注册表）与 JSON 持久化；
  5. **关于与环境（居中弹窗）**：实时读取渲染后端、当前缩放倍率、`pixels_per_point`、视口逻辑/物理尺寸与系统托盘状态。
- **统一的交互与无障碍规范**
  - **文本可选统一原则**：所有按钮、导航项、分段切换、开关、下拉框、滑动条与可点击列表行禁止拖选复制；其余全部展示性文字均支持鼠标拖选与 `Ctrl+C` 复制，选区配色满足高对比度；
  - **全量 AccessKit -> Windows UI Automation 支持**：全部自绘控件显式注册中文无障碍名称、角色与状态，并配套 13 项自动化 UIA、1:9 分栏比例、居中弹窗、`.lnk` 启动项、几何防重叠与最小窗口（`820×540`）回归测试。
- **完整的 Windows 桌面工程闭环**
  - 自定义 `winit` 隐藏窗口事件循环包装器（`src/event_loop.rs`），窗口隐藏到托盘或最小化时零轮询空转且即时响应托盘「退出」「打开主窗口」与单实例唤醒；
  - Win32 原生托盘（`NOTIFYICON_VERSION_4` + `WS_EX_TOPMOST` + `WM_CONTEXTMENU` + KB135788），确保 Windows 11 右键托盘菜单始终位于任务栏前方；
  - 基于命名 `CreateMutexW` 的单实例排他与重复启动自动唤回已有窗口。

---

## 目录结构

```text
UiShowcase/
├── Cargo.toml                         # Rust 工程配置（eframe 0.33 / glow / accesskit / windows）
├── build.rs                           # 编译期嵌入多分辨率应用图标与 PerMonitorV2 清单
├── assets/
│   ├── app.ico                        # 多分辨率应用图标
│   └── fonts/FZGWKaiS.ttf             # 内嵌方正公文楷体
├── src/
│   ├── main.rs                        # 单实例排他入口与窗口启动配置
│   ├── event_loop.rs                  # Windows 隐藏窗口事件循环包装器
│   ├── app.rs                         # 三内容页 + 统一居中弹窗 + 1:9 分栏主应用
│   ├── theme.rs                       # 语义双主题色板、四组强调色、双密度与字体管理
│   ├── widgets.rs                     # 全套自绘现代控件、居中弹窗容器与 AccessKit 语义
│   ├── system.rs                      # Win32 原生托盘、DWM 标题栏、系统主题与启动文件夹 .lnk 自启
│   └── settings.rs                    # 本地 JSON 配置持久化与隔离环境支持
├── Tests/
│   └── Verify-UiAutomation.ps1        # 13 项 UI Automation、1:9 分栏、弹窗、.lnk 与托盘回归脚本
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

# 运行 13 项 UI Automation、DPI、1:9 分栏、弹窗、.lnk 自启、持久化、单实例与托盘回归测试
powershell -NoProfile -ExecutionPolicy Bypass -File .\Tests\Verify-UiAutomation.ps1
```

构建产物位于 `target\release\UiShowcase.exe`。
