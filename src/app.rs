use crate::settings::{
    AccentPreset, AppSettings, DensityMode, FontChoice, ThemeMode, load_settings, save_settings,
    settings_path,
};
use crate::system::{
    TrayController, TrayEvent, apply_titlebar_theme, get_autostart_command, is_autostart_enabled,
    load_icon_rgba_for_texture, set_autostart_enabled,
};
use crate::theme::{DensityMetrics, Palette, apply_theme, hex_color, resolve_palette, setup_fonts};
use crate::widgets::{
    ButtonKind, StatusKind, nav_item, segment_bar, status_badge, status_banner, styled_button,
    surface_card, toggle_switch, workbench_task_header, workbench_task_row,
};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, ViewportCommand};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Workbench,
    Controls,
    Settings,
    About,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Page::Overview => "总览与色板",
            Page::Workbench => "典型工具界面",
            Page::Controls => "控件与交互态",
            Page::Settings => "主题与设置",
            Page::About => "关于与环境",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Page::Overview => "展示「澄明原生工具风 · Rust + egui」语义色板、排版层级与 OpenGL 原生清晰渲染",
            Page::Workbench => "模拟真实 Windows 桌面核查工具工作台：工具栏、列表、详情面板、任务进度与空状态",
            Page::Controls => "完整演示五态按钮、内联校验错误、开关、分段选择、下拉框、滑块及键盘焦点外描边",
            Page::Settings => "实时切换字体、主题模式、强调色与密度，管理托盘行为与登录时启动（真实持久化）",
            Page::About => "统一中文标识、版本号与当前 Rust + egui OpenGL (glow) / DPI 渲染环境读数",
        }
    }
}

#[derive(Debug, Clone)]
struct SampleTask {
    id: &'static str,
    file_name: &'static str,
    folder: &'static str,
    size: &'static str,
    modified: &'static str,
    status: &'static str,
    kind: StatusKind,
    sha: &'static str,
    note: &'static str,
}

fn initial_tasks() -> Vec<SampleTask> {
    vec![
        SampleTask {
            id: "DOC-001",
            file_name: "2026年度项目验收汇总表.xlsx",
            folder: r"D:\Codex\工作档案\验收材料",
            size: "  1.42 MB",
            modified: "2026-11-24 09:18",
            status: "已通过",
            kind: StatusKind::Success,
            sha: "A8F3-91C2",
            note: "结构与字段校验全部通过，可安全归档。",
        },
        SampleTask {
            id: "DOC-002",
            file_name: "无障碍测试记录_主流程.json",
            folder: r"D:\Codex\UiShowcase\Tests",
            size: " 48.60 KB",
            modified: "2026-11-24 11:04",
            status: "已通过",
            kind: StatusKind::Success,
            sha: "4C19-DE07",
            note: "AccessKit / UI Automation 节点名称、角色与键盘路径全部完整。",
        },
        SampleTask {
            id: "DOC-003",
            file_name: "桌面应用配置备份_旧版.ini",
            folder: r"D:\Codex\归档配置\2025",
            size: " 12.15 KB",
            modified: "2026-11-22 17:40",
            status: "需复核",
            kind: StatusKind::Warning,
            sha: "7B2E-04FA",
            note: "检测到 2 个旧版配置键，建议核对后再执行迁移。",
        },
        SampleTask {
            id: "DOC-004",
            file_name: "扫描件识别缓存_重复副本.pdf",
            folder: r"D:\Codex\临时导入\待清理",
            size: " 18.75 MB",
            modified: "2026-11-20 14:22",
            status: "存在异常",
            kind: StatusKind::Danger,
            sha: "F05D-88B1",
            note: "与归档目录中文件哈希完全一致，且缺少可提取文本层。",
        },
        SampleTask {
            id: "DOC-005",
            file_name: "系统托盘与启动项规范说明.md",
            folder: r"D:\Codex\_specs",
            size: "  8.94 KB",
            modified: "2026-11-25 12:20",
            status: "已通过",
            kind: StatusKind::Success,
            sha: "19EA-6B3C",
            note: "UTF-8 编码正常，标题层级与链接完整。",
        },
    ]
}

pub struct ShowcaseApp {
    settings: AppSettings,
    autostart_enabled: bool,
    autostart_command: String,
    settings_feedback: String,
    settings_has_error: bool,

    page: Page,
    status_message: String,
    icon_texture: Option<egui::TextureHandle>,
    hwnd_raw: isize,
    last_dark_applied: Option<bool>,
    exit_requested: bool,

    tray: TrayController,
    tray_rx: mpsc::Receiver<TrayEvent>,

    // 典型工具界面状态
    tasks: Vec<SampleTask>,
    selected_task_id: Option<String>,
    search_query: String,
    status_filter: &'static str,
    simulate_empty: bool,
    scanning: bool,
    scan_progress: f32,
    scan_last_tick: Option<Instant>,
    scan_status_text: String,
    confirm_delete_id: Option<String>,

    // 控件演示状态
    demo_path: String,
    demo_toggle_a: bool,
    demo_toggle_b: bool,
    demo_segment: usize,
    demo_slider: f32,
    demo_combo: &'static str,
    demo_feedback: String,
}

impl ShowcaseApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tray: TrayController,
        tray_rx: mpsc::Receiver<TrayEvent>,
    ) -> Self {
        let (settings, load_err) = load_settings();
        setup_fonts(&cc.egui_ctx, settings.font_choice);

        let palette = resolve_palette(settings.theme_mode, settings.accent_preset);
        let metrics = DensityMetrics::from_mode(settings.density_mode, settings.font_choice);
        apply_theme(&cc.egui_ctx, palette, metrics);

        tray.bind_context(cc.egui_ctx.clone());

        let hwnd_raw = cc
            .window_handle()
            .ok()
            .and_then(|h| match h.as_raw() {
                RawWindowHandle::Win32(w) => Some(w.hwnd.get()),
                _ => None,
            })
            .unwrap_or(0);
        apply_titlebar_theme(hwnd_raw, palette.dark);

        let icon_texture = load_icon_rgba_for_texture().map(|(w, h, rgba)| {
            let color_img = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);
            cc.egui_ctx
                .load_texture("app_icon", color_img, egui::TextureOptions::LINEAR)
        });

        let tasks = initial_tasks();
        let first_id = tasks.first().map(|t| t.id.to_owned());
        let autostart_enabled = is_autostart_enabled();
        let autostart_command = get_autostart_command();

        let (settings_feedback, settings_has_error) = match load_err {
            Some(err) => (err, true),
            None => ("已从本地配置加载真实状态。".to_owned(), false),
        };

        Self {
            settings,
            autostart_enabled,
            autostart_command,
            settings_feedback,
            settings_has_error,
            page: Page::Overview,
            status_message: "就绪 · Rust + egui（OpenGL / glow + AccessKit）原生渲染已启动"
                .to_owned(),
            icon_texture,
            hwnd_raw,
            last_dark_applied: Some(palette.dark),
            exit_requested: false,
            tray,
            tray_rx,
            tasks,
            selected_task_id: first_id,
            search_query: String::new(),
            status_filter: "全部状态",
            simulate_empty: false,
            scanning: false,
            scan_progress: 0.0,
            scan_last_tick: None,
            scan_status_text: "空闲".to_owned(),
            confirm_delete_id: None,
            demo_path: r"D:\Codex\UiShowcase".to_owned(),
            demo_toggle_a: true,
            demo_toggle_b: false,
            demo_segment: 0,
            demo_slider: 65.0,
            demo_combo: "标准校验（推荐）",
            demo_feedback: "点击下方任意按钮，或按 Tab 键体验统一 2px 键盘焦点外描边。".to_owned(),
        }
    }

    fn apply_visuals_and_save(&mut self, ctx: &egui::Context, reason: &str, reload_font: bool) {
        if reload_font {
            setup_fonts(ctx, self.settings.font_choice);
        }
        let p = resolve_palette(self.settings.theme_mode, self.settings.accent_preset);
        let m = DensityMetrics::from_mode(self.settings.density_mode, self.settings.font_choice);
        apply_theme(ctx, p, m);
        if self.last_dark_applied != Some(p.dark) {
            apply_titlebar_theme(self.hwnd_raw, p.dark);
            self.last_dark_applied = Some(p.dark);
        }
        match save_settings(&self.settings) {
            Ok(()) => {
                self.settings_feedback = format!("{reason}（已持久化保存至本地配置）");
                self.settings_has_error = false;
                self.status_message = format!(
                    "{reason} · 当前：{} / {}",
                    if p.dark { "深色模式" } else { "浅色模式" },
                    if self.settings.density_mode == DensityMode::Compact {
                        "紧凑密度"
                    } else {
                        "标准密度"
                    }
                );
            }
            Err(err) => {
                self.settings_feedback = err.clone();
                self.settings_has_error = true;
                self.status_message = err;
            }
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context) {
        self.exit_requested = true;
        // 不在 update() 中阻塞等待托盘线程退出；主事件循环关闭后由 on_exit()/Drop 清理托盘图标与线程
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    fn process_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.tray_rx.try_recv() {
            match ev {
                TrayEvent::OpenMain => {
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                    self.status_message = "已从系统托盘恢复主窗口".to_owned();
                }
                TrayEvent::OpenSettings => {
                    self.page = Page::Settings;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                    self.status_message = "已从托盘打开「主题与设置」".to_owned();
                }
                TrayEvent::ToggleTheme => {
                    let cur = resolve_palette(self.settings.theme_mode, self.settings.accent_preset);
                    self.settings.theme_mode = if cur.dark {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    };
                    self.apply_visuals_and_save(ctx, "主题模式已切换", false);
                }
                TrayEvent::Exit => {
                    self.request_exit(ctx);
                }
            }
        }
    }

    fn tick_scan_if_running(&mut self, ctx: &egui::Context) {
        if !self.scanning {
            return;
        }
        let now = Instant::now();
        let should_step = self
            .scan_last_tick
            .is_none_or(|t| now.duration_since(t) >= Duration::from_millis(160));
        if should_step {
            self.scan_last_tick = Some(now);
            self.scan_progress = (self.scan_progress + 0.1).min(1.0);
            let pct = (self.scan_progress * 100.0).round() as i32;
            if pct >= 100 {
                self.scanning = false;
                self.scan_status_text = format!("已完成核查 · 共 {} 项", self.tasks.len());
                self.status_message = "模拟核查完成：后台任务期间界面保持流畅可响应".to_owned();
            } else {
                self.scan_status_text = format!("正在核查文件哈希（{pct}%）…");
                ctx.request_repaint_after(Duration::from_millis(160));
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(80));
        }
    }

    fn demo_path_error(&self) -> Option<&'static str> {
        let trimmed = self.demo_path.trim();
        if trimmed.is_empty() {
            return Some("工作目录不能为空，请输入有效的本地文件夹路径。");
        }
        if trimmed
            .chars()
            .any(|c| matches!(c, '<' | '>' | '"' | '|' | '?' | '*'))
        {
            return Some("路径包含非法字符（< > \" | ? *），请修正后再继续。");
        }
        if !trimmed.contains(':') && !trimmed.starts_with(r"\\") {
            return Some(r"请填写完整绝对路径，例如 D:\Codex\UiShowcase。");
        }
        None
    }

    fn render_sidebar(&mut self, ctx: &egui::Context, p: Palette, m: DensityMetrics) {
        egui::SidePanel::left("main_nav")
            .exact_width(m.nav_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(p.bg_canvas)
                    .stroke(Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(14, 18)),
            )
            .show(ctx, |ui| {
                // 右侧 1px 细分隔线
                let panel_rect = ui.max_rect().expand2(egui::vec2(14.0, 18.0));
                ui.painter().line_segment(
                    [panel_rect.right_top(), panel_rect.right_bottom()],
                    Stroke::new(1.0, p.border_subtle),
                );

                // 品牌区
                ui.horizontal(|ui| {
                    if let Some(tex) = &self.icon_texture {
                        ui.add(egui::Image::new((tex.id(), egui::vec2(28.0, 28.0))));
                    }
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(
                            RichText::new("澄明界面展示")
                                .size(m.h3_size + 1.0)
                                .color(p.text_primary),
                        );
                        ui.label(
                            RichText::new("Rust + egui 原生工具风")
                                .size(m.caption_size)
                                .color(p.text_muted),
                        );
                    });
                });

                ui.add_space(20.0);

                for (pg, label) in [
                    (Page::Overview, "总览与色板"),
                    (Page::Workbench, "典型工具界面"),
                    (Page::Controls, "控件与交互态"),
                ] {
                    if nav_item(ui, p, m, label, self.page == pg).clicked() {
                        self.page = pg;
                        self.status_message = format!("已切换到「{}」", pg.title());
                    }
                    ui.add_space(2.0);
                }

                ui.add_space(8.0);
                let (sep_rect, _) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [sep_rect.left_center(), sep_rect.right_center()],
                    Stroke::new(1.0, p.border_subtle),
                );
                ui.add_space(8.0);

                for (pg, label) in [(Page::Settings, "主题与设置"), (Page::About, "关于与环境")] {
                    if nav_item(ui, p, m, label, self.page == pg).clicked() {
                        self.page = pg;
                        self.status_message = format!("已切换到「{}」", pg.title());
                    }
                    ui.add_space(2.0);
                }

                // 底部提示卡片与退出按钮
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if styled_button(ui, p, m, "退出应用", ButtonKind::Secondary, true, true)
                        .clicked()
                    {
                        self.request_exit(ctx);
                    }
                    ui.add_space(10.0);
                    egui::Frame::new()
                        .fill(p.bg_surface)
                        .stroke(Stroke::new(1.0, p.border_subtle))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new("提示：按 Tab 键可体验统一 2px 键盘焦点外描边，Ctrl+Q 直接退出。")
                                    .size(m.caption_size)
                                    .color(p.text_muted),
                            );
                        });
                });
            });
    }

    fn render_header(&mut self, ctx: &egui::Context, p: Palette, m: DensityMetrics) {
        egui::TopBottomPanel::top("page_header")
            .frame(
                egui::Frame::new()
                    .fill(p.bg_surface)
                    .inner_margin(egui::Margin::symmetric(m.header_pad_x, m.header_pad_y)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect().expand2(egui::vec2(
                    f32::from(m.header_pad_x),
                    f32::from(m.header_pad_y),
                ));
                ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    Stroke::new(1.0, p.border_subtle),
                );

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(
                            RichText::new(self.page.title())
                                .size(m.h1_size)
                                .color(p.text_primary),
                        );
                        ui.label(
                            RichText::new(self.page.subtitle())
                                .size(m.secondary_size)
                                .color(p.text_secondary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if styled_button(
                            ui,
                            p,
                            m,
                            "切换浅/深色",
                            ButtonKind::Secondary,
                            true,
                            false,
                        )
                        .clicked()
                        {
                            self.settings.theme_mode = if p.dark {
                                ThemeMode::Light
                            } else {
                                ThemeMode::Dark
                            };
                            self.apply_visuals_and_save(ctx, "主题模式已更新", false);
                        }

                        ui.add_space(8.0);
                        let mut density = self.settings.density_mode;
                        if segment_bar(
                            ui,
                            p,
                            m,
                            &mut density,
                            &[
                                (DensityMode::Standard, "标准"),
                                (DensityMode::Compact, "紧凑"),
                            ],
                            "快速切换密度",
                        ) {
                            self.settings.density_mode = density;
                            self.apply_visuals_and_save(ctx, "界面密度已更新", false);
                        }

                        ui.add_space(8.0);
                        let mut font = self.settings.font_choice;
                        if segment_bar(
                            ui,
                            p,
                            m,
                            &mut font,
                            &[
                                (FontChoice::FzGwKai, "楷体"),
                                (FontChoice::MicrosoftYaHei, "雅黑"),
                            ],
                            "快速切换字体",
                        ) {
                            self.settings.font_choice = font;
                            self.apply_visuals_and_save(ctx, "界面字体已切换", true);
                        }
                    });
                });
            });
    }

    fn render_status_bar(&self, ctx: &egui::Context, p: Palette, m: DensityMetrics) {
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(p.bg_surface)
                    .inner_margin(egui::Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect().expand2(egui::vec2(20.0, 8.0));
                ui.painter().line_segment(
                    [rect.left_top(), rect.right_top()],
                    Stroke::new(1.0, p.border_subtle),
                );
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.status_message)
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let font_label = match self.settings.font_choice {
                            FontChoice::FzGwKai => "方正公文楷体",
                            FontChoice::MicrosoftYaHei => "微软雅黑 UI",
                        };
                        let right = format!(
                            "{} · {} · {} · {:.0}% DPI",
                            if p.dark { "深色模式" } else { "浅色模式" },
                            if self.settings.density_mode == DensityMode::Compact {
                                "紧凑密度"
                            } else {
                                "标准密度"
                            },
                            font_label,
                            ctx.pixels_per_point() * 100.0
                        );
                        ui.label(RichText::new(right).size(m.caption_size).color(p.text_muted));
                    });
                });
            });
    }

    fn render_overview(&self, ui: &mut egui::Ui, p: Palette, m: DensityMetrics) {
        // 4 个核心原则卡片（严格 top_down 布局与列宽约束，绝不横向错乱）
        let avail_w = ui.available_width();
        let gap = 12.0;
        let card_w = ((avail_w - gap * 3.0) / 4.0).floor().max(160.0);
        let card_inner_w = (card_w - f32::from(m.card_pad) * 2.0).max(120.0);
        let card_min_h = 124.0 - f32::from(m.card_pad) * 2.0;

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (title, desc) in [
                (
                    "01 · GPU 锐利字体",
                    "基于 Rust + egui (OpenGL/glow) 按真实物理分辨率光栅化字形图集，高分屏无虚边。",
                ),
                (
                    "02 · 色阶与细线分层",
                    "常规内容区靠背景色差与 1px 细边框自然分区，全局无重阴影与浮夸渐变。",
                ),
                (
                    "03 · 语义双主题",
                    "全界面由语义 Token 驱动，一键切换浅色、深色、四组强调色与双密度档位。",
                ),
                (
                    "04 · 居中与无障碍",
                    "所有按钮与控件内置精确几何居中与 2px 焦点环，经 AccessKit 暴露完整 UIA 树。",
                ),
            ] {
                ui.allocate_ui_with_layout(
                    egui::vec2(card_w, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(card_w);
                        ui.set_max_width(card_w);
                        surface_card(ui, p, m, |ui| {
                            ui.set_width(card_inner_w);
                            ui.set_max_width(card_inner_w);
                            ui.set_min_height(card_min_h);
                            ui.label(RichText::new(title).size(m.h3_size).color(p.text_primary));
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(desc)
                                    .size(m.secondary_size)
                                    .color(p.text_secondary),
                            );
                        });
                    },
                );
            }
        });

        ui.add_space(m.section_gap);

        // 语义色板卡片
        surface_card(ui, p, m, |ui| {
            let panel_inner_w = ui.available_width();
            ui.set_width(panel_inner_w);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("语义色板（随浅色 / 深色与强调色实时刷新）")
                        .size(m.h2_size)
                        .color(p.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "当前生效：{}",
                            if p.dark { "深色模式" } else { "浅色模式" }
                        ))
                        .size(m.caption_size)
                        .color(p.text_muted),
                    );
                });
            });
            ui.add_space(12.0);

            let tokens: [(&str, &str, Color32, &str); 11] = [
                ("画布底色", "bg_canvas", p.bg_canvas, "窗口外层背景与侧边导航栏"),
                ("表面面板", "bg_surface", p.bg_surface, "主内容卡片、顶栏与列表容器"),
                ("控件底色", "bg_control", p.bg_control, "输入框、下拉框与次按钮默认底色"),
                ("选中底色", "bg_selected", p.bg_selected, "导航选中项与列表高亮行"),
                ("主文本", "text_primary", p.text_primary, "标题、正文与关键数据"),
                ("次要文本", "text_secondary", p.text_secondary, "描述文字、副标题与辅助说明"),
                ("细分隔线", "border_subtle", p.border_subtle, "1px 区域边界与内容分割线"),
                ("主强调色", "accent_primary", p.accent_primary, "主按钮、选中指示条与焦点环"),
                ("成功语义", "status_success", p.status_success, "校验通过、保存成功正向反馈"),
                ("警告语义", "status_warning", p.status_warning, "需人工复核、兼容提示状态"),
                ("错误与危险", "status_danger", p.status_danger, "内联报错、删除等不可逆警示"),
            ];

            let sw_gap = 12.0;
            let cols = ((panel_inner_w + sw_gap) / 260.0).floor().clamp(2.0, 4.0) as usize;
            let sw_w = ((panel_inner_w - sw_gap * (cols as f32 - 1.0)) / cols as f32).floor();
            let sw_inner_w = (sw_w - 24.0).max(160.0);
            let sw_text_w = (sw_inner_w - 38.0).max(110.0);

            for row in tokens.chunks(cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = sw_gap;
                    for &(name, token, color, usage) in row {
                        ui.allocate_ui_with_layout(
                            egui::vec2(sw_w, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_width(sw_w);
                                ui.set_max_width(sw_w);
                                egui::Frame::new()
                                    .fill(p.bg_canvas)
                                    .stroke(Stroke::new(1.0, p.border_subtle))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::same(12))
                                    .show(ui, |ui| {
                                        ui.set_width(sw_inner_w);
                                        ui.set_max_width(sw_inner_w);
                                        ui.set_min_height(66.0);
                                        ui.horizontal_top(|ui| {
                                            ui.spacing_mut().item_spacing.x = 10.0;
                                            let (box_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(28.0, 28.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().rect(
                                                box_rect,
                                                CornerRadius::same(4),
                                                color,
                                                Stroke::new(1.0, p.border_control),
                                                egui::StrokeKind::Inside,
                                            );
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(sw_text_w, 0.0),
                                                egui::Layout::top_down(egui::Align::LEFT),
                                                |ui| {
                                                    ui.set_width(sw_text_w);
                                                    ui.set_max_width(sw_text_w);
                                                    ui.spacing_mut().item_spacing.y = 3.0;
                                                    ui.horizontal(|ui| {
                                                        ui.set_width(sw_text_w);
                                                        ui.label(
                                                            RichText::new(name)
                                                                .size(m.body_size)
                                                                .color(p.text_primary),
                                                        );
                                                        ui.with_layout(
                                                            egui::Layout::right_to_left(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.label(
                                                                    RichText::new(hex_color(color))
                                                                        .monospace()
                                                                        .size(m.caption_size)
                                                                        .color(p.text_secondary),
                                                                );
                                                            },
                                                        );
                                                    });
                                                    ui.label(
                                                        RichText::new(token)
                                                            .monospace()
                                                            .size(m.caption_size)
                                                            .color(p.accent_primary),
                                                    );
                                                    ui.label(
                                                        RichText::new(usage)
                                                            .size(m.caption_size)
                                                            .color(p.text_muted),
                                                    );
                                                },
                                            );
                                        });
                                    });
                            },
                        );
                    }
                });
                ui.add_space(10.0);
            }
        });

        ui.add_space(m.section_gap);

        // 排版层级 + 数据对齐对照（严格双列 top_down + 等宽约束）
        let pair_gap = 14.0;
        let half_w = ((ui.available_width() - pair_gap) / 2.0).floor().max(260.0);
        let half_inner_w = (half_w - f32::from(m.card_pad) * 2.0).max(200.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = pair_gap;
            ui.allocate_ui_with_layout(
                egui::vec2(half_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(half_w);
                    ui.set_max_width(half_w);
                    surface_card(ui, p, m, |ui| {
                        ui.set_width(half_inner_w);
                        ui.set_max_width(half_inner_w);
                        ui.set_min_height(220.0);
                        let font_name = match self.settings.font_choice {
                            FontChoice::FzGwKai => "方正公文楷体（内嵌 TTF · GPU 图集）",
                            FontChoice::MicrosoftYaHei => "微软雅黑 UI（系统字体 · GPU 图集）",
                        };
                        ui.label(
                            RichText::new(format!("字体层级 · {font_name}"))
                                .size(m.h2_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!("H1 页面主标题 · {:.0}pt", m.h1_size))
                                .size(m.h1_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("H2 分区标题 · {:.0}pt", m.h2_size))
                                .size(m.h2_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("H3 卡片标题 · {:.0}pt", m.h3_size))
                                .size(m.h3_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Body 正文文字：由 egui 按当前屏幕缩放倍率直接生成高分字形图集，右上角可随时一键对比「楷体 / 雅黑」。")
                                .size(m.body_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Caption 辅助注释与状态栏文字")
                                .size(m.caption_size)
                                .color(p.text_muted),
                        );
                    });
                },
            );

            ui.allocate_ui_with_layout(
                egui::vec2(half_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(half_w);
                    ui.set_max_width(half_w);
                    surface_card(ui, p, m, |ui| {
                        ui.set_width(half_inner_w);
                        ui.set_max_width(half_inner_w);
                        ui.set_min_height(220.0);
                        ui.label(
                            RichText::new("数据与路径排版展示")
                                .size(m.h2_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("数值、哈希、路径与时间戳在列表与日志面板中保持清晰对齐：")
                                .size(m.secondary_size)
                                .color(p.text_secondary),
                        );
                        ui.add_space(10.0);
                        egui::Frame::new()
                            .fill(p.bg_canvas)
                            .stroke(Stroke::new(1.0, p.border_subtle))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(
                                        RichText::new("编号       大小        校验摘要    更新时间")
                                            .monospace()
                                            .size(m.secondary_size)
                                            .color(p.text_secondary),
                                    );
                                    ui.separator();
                                    for line in [
                                        "DOC-001      1.42 MB   A8F3-91C2   2026-11-24 09:18",
                                        "DOC-002     48.60 KB   4C19-DE07   2026-11-24 11:04",
                                        "DOC-003     12.15 KB   7B2E-04FA   2026-11-22 17:40",
                                        "DOC-004     18.75 MB   F05D-88B1   2026-11-20 14:22",
                                    ] {
                                        ui.label(
                                            RichText::new(line)
                                                .monospace()
                                                .size(m.secondary_size)
                                                .color(p.text_primary),
                                        );
                                    }
                                });
                            });
                    });
                },
            );
        });
    }

    fn render_workbench(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: Palette, m: DensityMetrics) {
        // 顶部工具栏（同一操作区最多 1 个主按钮）
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if styled_button(
                    ui,
                    p,
                    m,
                    "开始模拟核查",
                    ButtonKind::Primary,
                    !self.scanning,
                    false,
                )
                .clicked()
                {
                    self.simulate_empty = false;
                    self.scanning = true;
                    self.scan_progress = 0.0;
                    self.scan_last_tick = Some(Instant::now());
                    self.scan_status_text = "正在核查文件哈希（0%）…".to_owned();
                    self.status_message = "模拟后台核查已启动，界面保持可交互".to_owned();
                    ctx.request_repaint_after(Duration::from_millis(160));
                }

                if styled_button(
                    ui,
                    p,
                    m,
                    "取消任务",
                    ButtonKind::Secondary,
                    self.scanning,
                    false,
                )
                .clicked()
                {
                    self.scanning = false;
                    self.scan_status_text =
                        format!("已在 {:.0}% 处取消", self.scan_progress * 100.0);
                    self.status_message = "已取消模拟核查任务".to_owned();
                }

                let empty_btn_label = if self.simulate_empty {
                    "恢复示例列表"
                } else {
                    "预览空状态"
                };
                if styled_button(
                    ui,
                    p,
                    m,
                    empty_btn_label,
                    ButtonKind::Secondary,
                    true,
                    false,
                )
                .clicked()
                {
                    self.simulate_empty = !self.simulate_empty;
                    self.status_message = if self.simulate_empty {
                        "已切换为空状态规范演示".to_owned()
                    } else {
                        "已恢复示例任务列表".to_owned()
                    };
                }

                if styled_button(ui, p, m, "重置数据", ButtonKind::Ghost, true, false).clicked() {
                    self.tasks = initial_tasks();
                    self.selected_task_id = self.tasks.first().map(|t| t.id.to_owned());
                    self.search_query.clear();
                    self.status_filter = "全部状态";
                    self.simulate_empty = false;
                    self.status_message = "已重置典型工具界面演示数据".to_owned();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let search_edit = egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("搜索文件名、路径或编号…")
                        .desired_width(210.0)
                        .margin(egui::vec2(10.0, 7.0));
                    let resp = ui.add(search_edit);
                    resp.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            true,
                            "搜索核查记录",
                        )
                    });

                    egui::ComboBox::from_id_salt("workbench_status_filter")
                        .selected_text(self.status_filter)
                        .width(116.0)
                        .show_ui(ui, |ui| {
                            for opt in ["全部状态", "已通过", "需复核", "存在异常"] {
                                ui.selectable_value(&mut self.status_filter, opt, opt);
                            }
                        });
                });
            });
        });

        ui.add_space(m.row_gap);

        // 筛选后列表
        let q = self.search_query.trim().to_lowercase();
        let filtered: Vec<SampleTask> = self
            .tasks
            .iter()
            .filter(|t| {
                (self.status_filter == "全部状态" || t.status == self.status_filter)
                    && (q.is_empty()
                        || t.file_name.to_lowercase().contains(&q)
                        || t.folder.to_lowercase().contains(&q)
                        || t.id.to_lowercase().contains(&q))
            })
            .cloned()
            .collect();

        if self
            .selected_task_id
            .as_ref()
            .is_none_or(|id| !filtered.iter().any(|t| t.id == id))
        {
            self.selected_task_id = filtered.first().map(|t| t.id.to_owned());
        }

        // 进度汇总条
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let danger_count = self
                    .tasks
                    .iter()
                    .filter(|t| t.kind == StatusKind::Danger)
                    .count();
                ui.label(
                    RichText::new(format!(
                        "共 {} 项 · 当前筛选 {} 项 · 异常 {} 项",
                        self.tasks.len(),
                        filtered.len(),
                        danger_count
                    ))
                    .size(m.secondary_size)
                    .color(p.text_secondary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(&self.scan_status_text)
                            .monospace()
                            .size(m.caption_size)
                            .color(p.text_secondary),
                    );
                    ui.add_space(10.0);
                    let bar = egui::ProgressBar::new(self.scan_progress)
                        .desired_width(180.0)
                        .desired_height(8.0)
                        .fill(p.accent_primary);
                    ui.add(bar);
                });
            });
        });

        ui.add_space(m.row_gap);

        if self.simulate_empty || filtered.is_empty() {
            surface_card(ui, p, m, |ui| {
                ui.set_min_size(egui::vec2(ui.available_width(), 300.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(56.0);
                    let (circle_rect, _) =
                        ui.allocate_exact_size(egui::vec2(52.0, 52.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(circle_rect.center(), 26.0, p.accent_subtle);
                    ui.painter().text(
                        circle_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "空",
                        egui::FontId::proportional(22.0),
                        p.accent_primary,
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("当前没有符合条件的核查记录")
                            .size(m.h2_size)
                            .color(p.text_primary),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("规范要求空状态必须说明为空原因，并直接给出明确的下一步操作按钮。")
                            .size(m.secondary_size)
                            .color(p.text_secondary),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 250.0).max(0.0) / 2.0);
                        if styled_button(
                            ui,
                            p,
                            m,
                            "恢复示例列表",
                            ButtonKind::Primary,
                            true,
                            false,
                        )
                        .clicked()
                        {
                            self.tasks = initial_tasks();
                            self.selected_task_id = self.tasks.first().map(|t| t.id.to_owned());
                            self.search_query.clear();
                            self.status_filter = "全部状态";
                            self.simulate_empty = false;
                            self.status_message = "已恢复示例任务列表".to_owned();
                        }
                        if styled_button(
                            ui,
                            p,
                            m,
                            "开始模拟核查",
                            ButtonKind::Secondary,
                            !self.scanning,
                            false,
                        )
                        .clicked()
                        {
                            self.simulate_empty = false;
                            self.scanning = true;
                            self.scan_progress = 0.0;
                            self.scan_last_tick = Some(Instant::now());
                            ctx.request_repaint_after(Duration::from_millis(160));
                        }
                    });
                });
            });
            return;
        }

        // 左右分栏：左侧四列核查记录表 + 右侧选中项详情面板（严格限宽不重叠）
        let avail_w = ui.available_width();
        let gap = 16.0;
        let right_w = ((avail_w - gap) * 0.36).clamp(280.0, 380.0).floor();
        let left_w = (avail_w - gap - right_w).max(400.0).floor();

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;

            // 左侧列表卡片
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(left_w);
                    ui.set_max_width(left_w);
                    surface_card(ui, p, m, |ui| {
                        let inner_w = (left_w - f32::from(m.card_pad) * 2.0).max(320.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);

                        workbench_task_header(ui, p, m, inner_w);
                        ui.add_space(8.0);

                        for item in &filtered {
                            let is_selected =
                                self.selected_task_id.as_deref() == Some(item.id);
                            if workbench_task_row(
                                ui,
                                p,
                                m,
                                inner_w,
                                item.id,
                                item.file_name,
                                item.folder,
                                item.size,
                                item.status,
                                item.kind,
                                is_selected,
                            )
                            .clicked()
                            {
                                self.selected_task_id = Some(item.id.to_owned());
                            }
                            ui.add_space(6.0);
                        }
                    });
                },
            );

            // 右侧详情面板
            ui.allocate_ui_with_layout(
                egui::vec2(right_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(right_w);
                    ui.set_max_width(right_w);
                    surface_card(ui, p, m, |ui| {
                        let inner_w = (right_w - f32::from(m.card_pad) * 2.0).max(220.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);
                        if let Some(sel) = filtered
                            .iter()
                            .find(|t| self.selected_task_id.as_deref() == Some(t.id))
                        {
                            ui.label(
                                RichText::new("选中记录详情")
                                    .size(m.caption_size)
                                    .color(p.text_muted),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(sel.file_name)
                                    .size(m.h2_size)
                                    .color(p.text_primary),
                            );
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.label(RichText::new("记录编号").size(m.caption_size).color(p.text_muted));
                            ui.label(
                                RichText::new(sel.id)
                                    .monospace()
                                    .size(m.secondary_size)
                                    .color(p.text_primary),
                            );
                            ui.add_space(8.0);

                            ui.label(RichText::new("所在目录").size(m.caption_size).color(p.text_muted));
                            ui.label(
                                RichText::new(sel.folder)
                                    .monospace()
                                    .size(m.secondary_size)
                                    .color(p.text_primary),
                            );
                            ui.add_space(8.0);

                            // 文件大小与哈希摘要并排
                            let half = ((inner_w - 12.0) * 0.5).max(90.0);
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = 12.0;
                                ui.allocate_ui_with_layout(
                                    egui::vec2(half, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.set_width(half);
                                        ui.label(
                                            RichText::new("文件大小")
                                                .size(m.caption_size)
                                                .color(p.text_muted),
                                        );
                                        ui.label(
                                            RichText::new(sel.size.trim())
                                                .monospace()
                                                .size(m.secondary_size)
                                                .color(p.text_primary),
                                        );
                                    },
                                );
                                ui.allocate_ui_with_layout(
                                    egui::vec2(half, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.set_width(half);
                                        ui.label(
                                            RichText::new("哈希摘要")
                                                .size(m.caption_size)
                                                .color(p.text_muted),
                                        );
                                        ui.label(
                                            RichText::new(sel.sha)
                                                .monospace()
                                                .size(m.secondary_size)
                                                .color(p.text_primary),
                                        );
                                    },
                                );
                            });
                            ui.add_space(8.0);

                            ui.label(RichText::new("修改时间").size(m.caption_size).color(p.text_muted));
                            ui.label(
                                RichText::new(sel.modified)
                                    .monospace()
                                    .size(m.secondary_size)
                                    .color(p.text_primary),
                            );
                            ui.add_space(10.0);

                            status_banner(
                                ui,
                                p,
                                m,
                                sel.kind,
                                &format!("核查结论：{}", sel.status),
                                sel.note,
                            );
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(10.0);
                            if styled_button(
                                ui,
                                p,
                                m,
                                "移除选中记录（演示二次确认）",
                                ButtonKind::Danger,
                                true,
                                true,
                            )
                            .clicked()
                            {
                                self.confirm_delete_id = Some(sel.id.to_owned());
                            }
                        }
                    });
                },
            );
        });
    }

    fn render_controls(&mut self, ui: &mut egui::Ui, p: Palette, m: DensityMetrics) {
        // 按钮五态
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("按钮层级与五态闭环（默认 / 悬停 / 按下 / 禁用 / 键盘焦点）")
                    .size(m.h2_size)
                    .color(p.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("同一操作区最多只保留一个主按钮；所有按钮均实现精确几何居中与完整无障碍语义。")
                    .size(m.secondary_size)
                    .color(p.text_secondary),
            );
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                for (label, kind, enabled) in [
                    ("主操作按钮", ButtonKind::Primary, true),
                    ("次操作按钮", ButtonKind::Secondary, true),
                    ("幽灵轻量按钮", ButtonKind::Ghost, true),
                    ("危险操作按钮", ButtonKind::Danger, true),
                    ("禁用态主按钮", ButtonKind::Primary, false),
                    ("禁用态次按钮", ButtonKind::Secondary, false),
                ] {
                    if styled_button(ui, p, m, label, kind, enabled, false).clicked() {
                        self.demo_feedback = format!("已触发「{label}」，状态栏已同步反馈。");
                        self.status_message = format!("控件演示：已执行「{label}」");
                    }
                }
            });

            ui.add_space(10.0);
            egui::Frame::new()
                .fill(p.bg_canvas)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(12, 9))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(&self.demo_feedback)
                            .size(m.secondary_size)
                            .color(p.text_secondary),
                    );
                });
        });

        ui.add_space(m.section_gap);

        // 输入与内联错误 + 开关/分段/滑块（严格双列 top_down 布局与列宽约束）
        let pair_gap = 14.0;
        let half_w = ((ui.available_width() - pair_gap) / 2.0).floor().max(280.0);
        let half_inner_w = (half_w - f32::from(m.card_pad) * 2.0).max(220.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = pair_gap;
            ui.allocate_ui_with_layout(
                egui::vec2(half_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(half_w);
                    ui.set_max_width(half_w);
                    surface_card(ui, p, m, |ui| {
                        ui.set_width(half_inner_w);
                        ui.set_max_width(half_inner_w);
                        ui.set_min_height(270.0);
                        ui.label(
                            RichText::new("输入框与内联校验错误")
                                .size(m.h2_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("校验错误紧贴控件下方以「[错误] + 中文原因」呈现，不单靠颜色区分。")
                                .size(m.secondary_size)
                                .color(p.text_secondary),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("目标工作目录")
                                .size(m.body_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        let edit = egui::TextEdit::singleline(&mut self.demo_path)
                            .hint_text(r"请输入绝对路径，例如 D:\Codex\UiShowcase")
                            .desired_width(half_inner_w - 22.0)
                            .margin(egui::vec2(10.0, 8.0));
                        let resp = ui.add(edit);
                        resp.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::TextEdit,
                                true,
                                "目标工作目录",
                            )
                        });

                        if let Some(err) = self.demo_path_error() {
                            ui.add_space(6.0);
                            status_banner(
                                ui,
                                p,
                                m,
                                StatusKind::Danger,
                                &format!("[错误] {err}"),
                                "",
                            );
                        }

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if styled_button(
                                ui,
                                p,
                                m,
                                "模拟非法输入",
                                ButtonKind::Secondary,
                                true,
                                false,
                            )
                            .clicked()
                            {
                                self.demo_path = "invalid*path?".to_owned();
                            }
                            if styled_button(
                                ui,
                                p,
                                m,
                                "恢复合法路径",
                                ButtonKind::Ghost,
                                true,
                                false,
                            )
                            .clicked()
                            {
                                self.demo_path = r"D:\Codex\UiShowcase".to_owned();
                            }
                        });

                        ui.add_space(14.0);
                        ui.label(
                            RichText::new("下拉选择（深浅色弹层全适配）")
                                .size(m.body_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_salt("demo_mode_combo")
                            .selected_text(self.demo_combo)
                            .width(half_inner_w)
                            .show_ui(ui, |ui| {
                                for opt in [
                                    "标准校验（推荐）",
                                    "严格哈希比对",
                                    "仅校验文件名与大小",
                                    "快速跳过大文件",
                                ] {
                                    ui.selectable_value(&mut self.demo_combo, opt, opt);
                                }
                            });
                    });
                },
            );

            ui.allocate_ui_with_layout(
                egui::vec2(half_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.set_width(half_w);
                    ui.set_max_width(half_w);
                    surface_card(ui, p, m, |ui| {
                        ui.set_width(half_inner_w);
                        ui.set_max_width(half_inner_w);
                        ui.set_min_height(270.0);
                        ui.label(
                            RichText::new("开关、分段选择与数值调节")
                                .size(m.h2_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(12.0);

                        if toggle_switch(
                            ui,
                            p,
                            m,
                            &mut self.demo_toggle_a,
                            "保存配置时自动校验完整性",
                        )
                        .changed()
                        {
                            self.status_message = format!(
                                "示例开关「保存配置时自动校验完整性」已{}",
                                if self.demo_toggle_a { "开启" } else { "关闭" }
                            );
                        }
                        ui.add_space(10.0);
                        if toggle_switch(
                            ui,
                            p,
                            m,
                            &mut self.demo_toggle_b,
                            "列表中仅展示存在异常的记录",
                        )
                        .changed()
                        {
                            self.status_message = format!(
                                "示例开关「列表中仅展示存在异常的记录」已{}",
                                if self.demo_toggle_b { "开启" } else { "关闭" }
                            );
                        }

                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("分段选择（互斥模式切换）")
                                .size(m.body_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(6.0);
                        segment_bar(
                            ui,
                            p,
                            m,
                            &mut self.demo_segment,
                            &[(0, "精确匹配"), (1, "模糊检索"), (2, "正则规则")],
                            "匹配模式",
                        );

                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.set_width(half_inner_w);
                            ui.label(
                                RichText::new("并发扫描阈值")
                                    .size(m.body_size)
                                    .color(p.text_primary),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(format!("{:.0}%", self.demo_slider))
                                            .monospace()
                                            .size(m.body_size)
                                            .color(p.accent_primary),
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        let slider = egui::Slider::new(&mut self.demo_slider, 10.0..=100.0)
                            .step_by(5.0)
                            .show_value(false);
                        ui.add_sized([half_inner_w, 26.0], slider);
                    });
                },
            );
        });

        ui.add_space(m.section_gap);

        // 状态反馈四件套（严格 2×2 top_down 网格，绝不重叠错乱）
        surface_card(ui, p, m, |ui| {
            let panel_inner_w = ui.available_width();
            ui.set_width(panel_inner_w);
            ui.label(
                RichText::new("状态反馈四件套（信息 / 成功 / 警告 / 错误）")
                    .size(m.h2_size)
                    .color(p.text_primary),
            );
            ui.add_space(12.0);
            let b_gap = 12.0;
            let bw = ((panel_inner_w - b_gap) / 2.0).floor().max(240.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = b_gap;
                ui.allocate_ui_with_layout(
                    egui::vec2(bw, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(bw);
                        ui.set_max_width(bw);
                        status_banner(
                            ui,
                            p,
                            m,
                            StatusKind::Info,
                            "[信息] OpenGL (glow) 按物理像素实时光栅化",
                            "字形图集与屏幕真实缩放倍率 1:1 匹配，100%/125%/150%/200% 均锐利清晰。",
                        );
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(bw, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(bw);
                        ui.set_max_width(bw);
                        status_banner(
                            ui,
                            p,
                            m,
                            StatusKind::Success,
                            "[成功] 本地设置已持久化保存",
                            "重启应用后自动恢复当前字体、主题模式、强调色、界面密度与托盘行为。",
                        );
                    },
                );
            });
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = b_gap;
                ui.allocate_ui_with_layout(
                    egui::vec2(bw, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(bw);
                        ui.set_max_width(bw);
                        status_banner(
                            ui,
                            p,
                            m,
                            StatusKind::Warning,
                            "[警告] 检测到 1 项需人工复核的旧版配置",
                            "警告色与品牌强调色彻底解耦，避免混淆正常高亮与风险提示。",
                        );
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(bw, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(bw);
                        ui.set_max_width(bw);
                        status_banner(
                            ui,
                            p,
                            m,
                            StatusKind::Danger,
                            "[错误] 目标目录不可写或权限不足",
                            "面向用户的系统错误须转换为可理解的简体中文说明与处理建议。",
                        );
                    },
                );
            });
        });
    }

    fn render_settings(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, p: Palette, m: DensityMetrics) {
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("外观与字体").size(m.h2_size).color(p.text_primary));
            ui.add_space(14.0);

            // 字体切换
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("界面字体").size(m.h3_size).color(p.text_primary));
                    ui.label(
                        RichText::new("默认按 egui-ui.md 嵌入方正公文楷体，可一键切微软雅黑 UI 实时对比")
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut font = self.settings.font_choice;
                    if segment_bar(
                        ui,
                        p,
                        m,
                        &mut font,
                        &[
                            (FontChoice::FzGwKai, "方正公文楷体"),
                            (FontChoice::MicrosoftYaHei, "微软雅黑 UI"),
                        ],
                        "界面字体",
                    ) {
                        self.settings.font_choice = font;
                        self.apply_visuals_and_save(ctx, "界面字体已切换", true);
                    }
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // 主题模式
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("主题模式").size(m.h3_size).color(p.text_primary));
                    ui.label(
                        RichText::new("支持浅色、深色以及跟随 Windows 系统外观，标题栏联动变色")
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut mode = self.settings.theme_mode;
                    if segment_bar(
                        ui,
                        p,
                        m,
                        &mut mode,
                        &[
                            (ThemeMode::System, "跟随系统"),
                            (ThemeMode::Light, "浅色模式"),
                            (ThemeMode::Dark, "深色模式"),
                        ],
                        "主题模式",
                    ) {
                        self.settings.theme_mode = mode;
                        self.apply_visuals_and_save(ctx, "主题模式已更新", false);
                    }
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // 强调色
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("品牌强调色").size(m.h3_size).color(p.text_primary));
                    ui.label(
                        RichText::new("仅替换 1 组强调色 Token，即可为不同工具建立清晰辨识度")
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut ac = self.settings.accent_preset;
                    if segment_bar(
                        ui,
                        p,
                        m,
                        &mut ac,
                        &[
                            (AccentPreset::Blue, "澄明蓝"),
                            (AccentPreset::Teal, "青瓷绿"),
                            (AccentPreset::Indigo, "暮山紫"),
                            (AccentPreset::Amber, "琥珀金"),
                        ],
                        "强调色",
                    ) {
                        self.settings.accent_preset = ac;
                        self.apply_visuals_and_save(ctx, "强调色方案已更新", false);
                    }
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // 界面密度
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("界面密度").size(m.h3_size).color(p.text_primary));
                    ui.label(
                        RichText::new("标准密度舒展从容；紧凑密度适合信息密集的列表与表格工具")
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut dens = self.settings.density_mode;
                    if segment_bar(
                        ui,
                        p,
                        m,
                        &mut dens,
                        &[
                            (DensityMode::Standard, "标准密度"),
                            (DensityMode::Compact, "紧凑密度"),
                        ],
                        "界面密度",
                    ) {
                        self.settings.density_mode = dens;
                        self.apply_visuals_and_save(ctx, "界面密度已更新", false);
                    }
                });
            });
        });

        ui.add_space(m.section_gap);

        // 托盘与开机自启
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("托盘与系统启动").size(m.h2_size).color(p.text_primary));
            ui.add_space(12.0);

            if toggle_switch(
                ui,
                p,
                m,
                &mut self.settings.close_to_tray,
                "关闭主窗口时隐藏到系统托盘（通过托盘菜单「退出」可彻底关闭）",
            )
            .changed()
            {
                self.apply_visuals_and_save(ctx, "关闭窗口行为已保存", false);
            }
            ui.add_space(10.0);

            let mut auto = self.autostart_enabled;
            if toggle_switch(
                ui,
                p,
                m,
                &mut auto,
                "登录时启动（同步写入当前用户启动项注册表）",
            )
            .changed()
            {
                match set_autostart_enabled(auto, self.settings.start_minimized_on_autostart) {
                    Ok(()) => {
                        self.autostart_enabled = is_autostart_enabled();
                        self.autostart_command = get_autostart_command();
                        let msg = if self.autostart_enabled {
                            "已启用登录时启动"
                        } else {
                            "已停用登录时启动"
                        };
                        self.settings_feedback = msg.to_owned();
                        self.settings_has_error = false;
                        self.status_message = msg.to_owned();
                    }
                    Err(err) => {
                        self.settings_feedback = err.clone();
                        self.settings_has_error = true;
                        self.status_message = err;
                    }
                }
            }
            ui.add_space(10.0);

            if toggle_switch(
                ui,
                p,
                m,
                &mut self.settings.start_minimized_on_autostart,
                "登录启动时默认隐藏到系统托盘（启动参数附带 --minimized）",
            )
            .changed()
            {
                self.apply_visuals_and_save(ctx, "登录启动显示行为已保存", false);
                if self.autostart_enabled {
                    let _ = set_autostart_enabled(true, self.settings.start_minimized_on_autostart);
                    self.autostart_command = get_autostart_command();
                }
            }

            ui.add_space(12.0);
            egui::Frame::new()
                .fill(p.bg_canvas)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("当前登录启动项注册状态：")
                            .size(m.caption_size)
                            .color(p.text_muted),
                    );
                    ui.label(
                        RichText::new(&self.autostart_command)
                            .monospace()
                            .size(m.secondary_size)
                            .color(p.text_primary),
                    );
                });
        });

        ui.add_space(m.section_gap);

        // 配置路径与重置
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("配置文件与恢复默认")
                            .size(m.h2_size)
                            .color(p.text_primary),
                    );
                    ui.label(
                        RichText::new(format!("存储位置：{}", settings_path().display()))
                            .monospace()
                            .size(m.caption_size)
                            .color(p.text_secondary),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(ui, p, m, "恢复默认设置", ButtonKind::Secondary, true, false)
                        .clicked()
                    {
                        self.settings = AppSettings::default();
                        self.apply_visuals_and_save(ctx, "已恢复默认外观与窗口设置", true);
                    }
                });
            });
            ui.add_space(10.0);
            status_banner(
                ui,
                p,
                m,
                if self.settings_has_error {
                    StatusKind::Danger
                } else {
                    StatusKind::Info
                },
                &self.settings_feedback,
                "",
            );
        });
    }

    fn render_about(&self, ctx: &egui::Context, ui: &mut egui::Ui, p: Palette, m: DensityMetrics) {
        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if let Some(tex) = &self.icon_texture {
                    ui.add(egui::Image::new((tex.id(), egui::vec2(56.0, 56.0))));
                }
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("澄明界面展示")
                                .size(m.h1_size)
                                .color(p.text_primary),
                        );
                        ui.add_space(8.0);
                        status_badge(
                            ui,
                            p,
                            m,
                            &format!("v{}", env!("CARGO_PKG_VERSION")),
                            StatusKind::Info,
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("基于 Rust + egui (eframe + OpenGL glow 后端 + AccessKit 无障碍桥接) 重写，彻底告别 WPF 在高分屏下的 ClearType 降级与模糊问题，原生支持方正公文楷体与微软雅黑 UI 实时切换。")
                            .size(m.body_size)
                            .color(p.text_secondary),
                    );
                });
            });
        });

        ui.add_space(m.section_gap);

        surface_card(ui, p, m, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("当前渲染与 DPI 环境读数")
                    .size(m.h2_size)
                    .color(p.text_primary),
            );
            ui.add_space(10.0);
            let ppp = ctx.pixels_per_point();
            let rect = ctx.viewport_rect();
            let metrics = format!(
                "渲染后端：Rust + eframe 0.33.3 + OpenGL (glow) + AccessKit\n\
                 当前缩放倍率：{:.0}%（pixels_per_point = {:.2}，物理 DPI = {:.0}）\n\
                 视口逻辑尺寸：{:.0} × {:.0} pt（物理渲染 {:.0} × {:.0} px）\n\
                 系统托盘状态：{}",
                ppp * 100.0,
                ppp,
                ppp * 96.0,
                rect.width(),
                rect.height(),
                rect.width() * ppp,
                rect.height() * ppp,
                if self.tray.is_registered() {
                    "已注册（澄明界面展示）"
                } else {
                    "当前桌面无 Shell_TrayWnd（独立测试桌面）"
                }
            );
            egui::Frame::new()
                .fill(p.bg_canvas)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(metrics)
                            .monospace()
                            .size(m.secondary_size)
                            .color(p.text_primary),
                    );
                });
        });
    }

    fn render_confirm_modal(&mut self, ctx: &egui::Context, p: Palette, m: DensityMetrics) {
        let Some(target_id) = self.confirm_delete_id.clone() else {
            return;
        };
        let target_name = self
            .tasks
            .iter()
            .find(|t| t.id == target_id)
            .map(|t| t.file_name)
            .unwrap_or("选中记录");

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.confirm_delete_id = None;
            self.status_message = "已取消移除操作".to_owned();
            return;
        }

        // 模态遮罩
        let screen = ctx.viewport_rect();
        egui::Area::new(egui::Id::new("confirm_scrim"))
            .fixed_pos(screen.min)
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                let (_, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter()
                    .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(110));
                if resp.clicked() {
                    self.confirm_delete_id = None;
                }
            });

        egui::Window::new("确认移除演示记录")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([420.0, 170.0])
            .order(egui::Order::Foreground)
            .frame(
                egui::Frame::new()
                    .fill(p.bg_surface)
                    .stroke(Stroke::new(1.0, p.border_control))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(22)),
            )
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "确定要从当前演示列表中移除「{target_name}」吗？\n\n此操作仅影响界面演示数据，不会删除磁盘上的任何真实文件。"
                    ))
                    .size(m.body_size)
                    .color(p.text_secondary),
                );
                ui.add_space(18.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(ui, p, m, "确认移除", ButtonKind::Danger, true, false)
                        .clicked()
                    {
                        self.tasks.retain(|t| t.id != target_id);
                        self.confirm_delete_id = None;
                        self.status_message = format!("已移除演示记录「{target_name}」");
                    }
                    ui.add_space(8.0);
                    if styled_button(ui, p, m, "取消", ButtonKind::Secondary, true, false)
                        .clicked()
                    {
                        self.confirm_delete_id = None;
                        self.status_message = "已取消移除操作".to_owned();
                    }
                });
            });
    }
}

impl eframe::App for ShowcaseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_tray_events(ctx);
        self.tick_scan_if_running(ctx);

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            self.request_exit(ctx);
        }

        if ctx.input(|i| i.viewport().close_requested())
            && self.settings.close_to_tray
            && self.tray.is_registered()
            && !self.exit_requested
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            self.status_message = "主窗口已隐藏到系统托盘，单击或右键托盘图标可重新打开".to_owned();
        }

        let p = resolve_palette(self.settings.theme_mode, self.settings.accent_preset);
        let m = DensityMetrics::from_mode(self.settings.density_mode, self.settings.font_choice);

        self.render_sidebar(ctx, p, m);
        self.render_header(ctx, p, m);
        self.render_status_bar(ctx, p, m);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.bg_canvas)
                    .inner_margin(egui::Margin::same(m.page_pad)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.page {
                        Page::Overview => self.render_overview(ui, p, m),
                        Page::Workbench => self.render_workbench(ctx, ui, p, m),
                        Page::Controls => self.render_controls(ui, p, m),
                        Page::Settings => self.render_settings(ctx, ui, p, m),
                        Page::About => self.render_about(ctx, ui, p, m),
                    });
            });

        self.render_confirm_modal(ctx, p, m);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.tray.shutdown();
    }
}
