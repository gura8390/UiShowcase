use crate::settings::{AccentPreset, DensityMode, FontChoice, ThemeMode};
use eframe::egui::{self, Color32, CornerRadius, Shadow, Stroke};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg_canvas: Color32,
    pub bg_surface: Color32,
    pub bg_control: Color32,
    pub bg_control_alt: Color32,
    pub bg_hover: Color32,
    pub bg_pressed: Color32,
    pub bg_selected: Color32,
    pub bg_disabled: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub text_on_accent: Color32,
    pub border_subtle: Color32,
    pub border_control: Color32,
    pub border_strong: Color32,
    pub accent_primary: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    pub accent_subtle: Color32,
    pub status_success: Color32,
    pub status_success_bg: Color32,
    pub status_success_border: Color32,
    pub status_warning: Color32,
    pub status_warning_bg: Color32,
    pub status_warning_border: Color32,
    pub status_danger: Color32,
    pub status_danger_btn: Color32,
    pub status_danger_hover: Color32,
    pub status_danger_bg: Color32,
    pub status_danger_border: Color32,
    pub status_info: Color32,
    pub status_info_bg: Color32,
    pub status_info_border: Color32,
    pub focus_ring: Color32,
}

#[derive(Debug, Clone, Copy)]
pub struct DensityMetrics {
    pub nav_width: f32,
    pub header_pad_x: i8,
    pub header_pad_y: i8,
    pub page_pad: i8,
    pub card_pad: i8,
    pub section_gap: f32,
    pub row_gap: f32,
    pub control_height: f32,
    pub nav_item_height: f32,
    pub body_size: f32,
    pub secondary_size: f32,
    pub caption_size: f32,
    pub h1_size: f32,
    pub h2_size: f32,
    pub h3_size: f32,
}

impl DensityMetrics {
    pub fn from_mode(mode: DensityMode, font: FontChoice) -> Self {
        let kai_bump = if font == FontChoice::FzGwKai { 1.0 } else { 0.0 };
        match mode {
            DensityMode::Standard => Self {
                nav_width: 232.0,
                header_pad_x: 26,
                header_pad_y: 16,
                page_pad: 24,
                card_pad: 20,
                section_gap: 20.0,
                row_gap: 12.0,
                control_height: 36.0,
                nav_item_height: 40.0,
                body_size: 14.0 + kai_bump,
                secondary_size: 13.0 + kai_bump,
                caption_size: 12.0 + kai_bump,
                h1_size: 24.0 + kai_bump,
                h2_size: 18.0 + kai_bump,
                h3_size: 15.0 + kai_bump,
            },
            DensityMode::Compact => Self {
                nav_width: 196.0,
                header_pad_x: 16,
                header_pad_y: 10,
                page_pad: 14,
                card_pad: 12,
                section_gap: 10.0,
                row_gap: 8.0,
                control_height: 28.0,
                nav_item_height: 32.0,
                body_size: 13.0 + kai_bump,
                secondary_size: 12.0 + kai_bump,
                caption_size: 11.5 + kai_bump,
                h1_size: 21.0 + kai_bump,
                h2_size: 16.0 + kai_bump,
                h3_size: 14.0 + kai_bump,
            },
        }
    }
}

pub fn hex_color(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

pub fn resolve_palette(theme: ThemeMode, accent: AccentPreset) -> Palette {
    let dark = match theme {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => crate::system::is_windows_dark_theme(),
    };

    let (ac_pri, ac_hov, ac_prs, ac_sub, bg_sel, on_ac) = match (accent, dark) {
        (AccentPreset::Blue, false) => (
            Color32::from_rgb(0x0F, 0x6C, 0xBD),
            Color32::from_rgb(0x11, 0x5E, 0xA3),
            Color32::from_rgb(0x0C, 0x4B, 0x85),
            Color32::from_rgb(0xEB, 0xF4, 0xFC),
            Color32::from_rgb(0xE6, 0xF1, 0xFB),
            Color32::WHITE,
        ),
        (AccentPreset::Blue, true) => (
            Color32::from_rgb(0x4E, 0xA8, 0xF5),
            Color32::from_rgb(0x6B, 0xB7, 0xF7),
            Color32::from_rgb(0x38, 0x92, 0xE0),
            Color32::from_rgb(0x1D, 0x32, 0x47),
            Color32::from_rgb(0x1F, 0x36, 0x4D),
            Color32::from_rgb(0x0D, 0x11, 0x17),
        ),
        (AccentPreset::Teal, false) => (
            Color32::from_rgb(0x0E, 0x7A, 0x6D),
            Color32::from_rgb(0x0B, 0x63, 0x59),
            Color32::from_rgb(0x08, 0x4E, 0x46),
            Color32::from_rgb(0xE7, 0xF6, 0xF3),
            Color32::from_rgb(0xE1, 0xF3, 0xF0),
            Color32::WHITE,
        ),
        (AccentPreset::Teal, true) => (
            Color32::from_rgb(0x46, 0xC2, 0xB2),
            Color32::from_rgb(0x62, 0xCF, 0xC1),
            Color32::from_rgb(0x33, 0xA6, 0x97),
            Color32::from_rgb(0x19, 0x35, 0x32),
            Color32::from_rgb(0x1D, 0x3D, 0x39),
            Color32::from_rgb(0x0B, 0x16, 0x15),
        ),
        (AccentPreset::Indigo, false) => (
            Color32::from_rgb(0x4F, 0x46, 0xE5),
            Color32::from_rgb(0x43, 0x38, 0xCA),
            Color32::from_rgb(0x37, 0x30, 0xA3),
            Color32::from_rgb(0xEE, 0xF0, 0xFE),
            Color32::from_rgb(0xE8, 0xEA, 0xFC),
            Color32::WHITE,
        ),
        (AccentPreset::Indigo, true) => (
            Color32::from_rgb(0x81, 0x8C, 0xF8),
            Color32::from_rgb(0x9C, 0xA5, 0xFA),
            Color32::from_rgb(0x63, 0x66, 0xF1),
            Color32::from_rgb(0x25, 0x26, 0x47),
            Color32::from_rgb(0x2B, 0x2D, 0x54),
            Color32::from_rgb(0x0F, 0x10, 0x20),
        ),
        (AccentPreset::Amber, false) => (
            Color32::from_rgb(0xB4, 0x53, 0x09),
            Color32::from_rgb(0x92, 0x40, 0x0E),
            Color32::from_rgb(0x78, 0x35, 0x0F),
            Color32::from_rgb(0xFE, 0xF5, 0xE7),
            Color32::from_rgb(0xFD, 0xEF, 0xD9),
            Color32::WHITE,
        ),
        (AccentPreset::Amber, true) => (
            Color32::from_rgb(0xF5, 0xB0, 0x41),
            Color32::from_rgb(0xF7, 0xC0, 0x65),
            Color32::from_rgb(0xD9, 0x95, 0x29),
            Color32::from_rgb(0x3B, 0x2D, 0x16),
            Color32::from_rgb(0x42, 0x32, 0x18),
            Color32::from_rgb(0x18, 0x12, 0x07),
        ),
    };

    if dark {
        Palette {
            dark: true,
            bg_canvas: Color32::from_rgb(0x18, 0x1A, 0x1F),
            bg_surface: Color32::from_rgb(0x22, 0x25, 0x2C),
            bg_control: Color32::from_rgb(0x2A, 0x2E, 0x37),
            bg_control_alt: Color32::from_rgb(0x1E, 0x21, 0x28),
            bg_hover: Color32::from_rgb(0x32, 0x37, 0x43),
            bg_pressed: Color32::from_rgb(0x3B, 0x41, 0x50),
            bg_selected: bg_sel,
            bg_disabled: Color32::from_rgb(0x20, 0x23, 0x2A),
            text_primary: Color32::from_rgb(0xF2, 0xF4, 0xF8),
            text_secondary: Color32::from_rgb(0xAE, 0xB8, 0xC6),
            text_muted: Color32::from_rgb(0x8C, 0x97, 0xA6),
            text_disabled: Color32::from_rgb(0x5C, 0x65, 0x73),
            text_on_accent: on_ac,
            border_subtle: Color32::from_rgb(0x2F, 0x34, 0x3F),
            border_control: Color32::from_rgb(0x43, 0x4B, 0x5B),
            border_strong: Color32::from_rgb(0x66, 0x72, 0x88),
            accent_primary: ac_pri,
            accent_hover: ac_hov,
            accent_pressed: ac_prs,
            accent_subtle: ac_sub,
            status_success: Color32::from_rgb(0x5C, 0xC8, 0x64),
            status_success_bg: Color32::from_rgb(0x1B, 0x2E, 0x1E),
            status_success_border: Color32::from_rgb(0x2C, 0x53, 0x32),
            status_warning: Color32::from_rgb(0xF2, 0xB8, 0x4B),
            status_warning_bg: Color32::from_rgb(0x33, 0x29, 0x14),
            status_warning_border: Color32::from_rgb(0x5E, 0x4A, 0x1E),
            status_danger: Color32::from_rgb(0xF1, 0x70, 0x7B),
            status_danger_btn: Color32::from_rgb(0xC4, 0x2B, 0x1C),
            status_danger_hover: Color32::from_rgb(0xD8, 0x3B, 0x2E),
            status_danger_bg: Color32::from_rgb(0x3A, 0x1E, 0x22),
            status_danger_border: Color32::from_rgb(0x69, 0x31, 0x39),
            status_info: Color32::from_rgb(0x5C, 0xB3, 0xFA),
            status_info_bg: Color32::from_rgb(0x1B, 0x2D, 0x40),
            status_info_border: Color32::from_rgb(0x2C, 0x4E, 0x70),
            focus_ring: ac_hov,
        }
    } else {
        Palette {
            dark: false,
            bg_canvas: Color32::from_rgb(0xF6, 0xF7, 0xF9),
            bg_surface: Color32::WHITE,
            bg_control: Color32::WHITE,
            bg_control_alt: Color32::from_rgb(0xF0, 0xF2, 0xF5),
            bg_hover: Color32::from_rgb(0xEE, 0xF1, 0xF5),
            bg_pressed: Color32::from_rgb(0xE2, 0xE7, 0xEE),
            bg_selected: bg_sel,
            bg_disabled: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            text_primary: Color32::from_rgb(0x14, 0x17, 0x1A),
            text_secondary: Color32::from_rgb(0x47, 0x50, 0x5A),
            text_muted: Color32::from_rgb(0x63, 0x6D, 0x78),
            text_disabled: Color32::from_rgb(0x9A, 0xA4, 0xB2),
            text_on_accent: on_ac,
            border_subtle: Color32::from_rgb(0xE5, 0xE8, 0xEC),
            border_control: Color32::from_rgb(0xCF, 0xD5, 0xDD),
            border_strong: Color32::from_rgb(0x9A, 0xA4, 0xB2),
            accent_primary: ac_pri,
            accent_hover: ac_hov,
            accent_pressed: ac_prs,
            accent_subtle: ac_sub,
            status_success: Color32::from_rgb(0x10, 0x7C, 0x10),
            status_success_bg: Color32::from_rgb(0xEA, 0xF6, 0xEC),
            status_success_border: Color32::from_rgb(0xB7, 0xDF, 0xBF),
            status_warning: Color32::from_rgb(0x9D, 0x5D, 0x00),
            status_warning_bg: Color32::from_rgb(0xFF, 0xF4, 0xCE),
            status_warning_border: Color32::from_rgb(0xEE, 0xD2, 0x7B),
            status_danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
            status_danger_btn: Color32::from_rgb(0xC4, 0x2B, 0x1C),
            status_danger_hover: Color32::from_rgb(0xA4, 0x26, 0x1A),
            status_danger_bg: Color32::from_rgb(0xFD, 0xE7, 0xE9),
            status_danger_border: Color32::from_rgb(0xF1, 0xB8, 0xBD),
            status_info: Color32::from_rgb(0x0F, 0x6C, 0xBD),
            status_info_bg: Color32::from_rgb(0xEB, 0xF4, 0xFC),
            status_info_border: Color32::from_rgb(0xB9, 0xD7, 0xF3),
            focus_ring: ac_pri,
        }
    }
}

pub fn setup_fonts(ctx: &egui::Context, choice: FontChoice) {
    let mut fonts = egui::FontDefinitions::default();

    // 静态嵌入方正公文楷体（符合 egui-ui.md）
    fonts.font_data.insert(
        "fzgwkai".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/FZGWKaiS.ttf"
        ))),
    );

    // 可选加载系统原生微软雅黑 UI 与 Consolas，便于在界面内实时对比效果
    let has_yahei = std::fs::read(r"C:\Windows\Fonts\msyh.ttc")
        .ok()
        .map(|bytes| {
            fonts
                .font_data
                .insert("msyh".to_owned(), Arc::new(egui::FontData::from_owned(bytes)));
        })
        .is_some();

    let has_mono = std::fs::read(r"C:\Windows\Fonts\consola.ttf")
        .ok()
        .map(|bytes| {
            fonts.font_data.insert(
                "consola".to_owned(),
                Arc::new(egui::FontData::from_owned(bytes)),
            );
        })
        .is_some();

    {
        let prop = fonts.families.entry(egui::FontFamily::Proportional).or_default();
        match choice {
            FontChoice::FzGwKai => {
                prop.insert(0, "fzgwkai".to_owned());
                if has_yahei {
                    prop.insert(1, "msyh".to_owned());
                }
            }
            FontChoice::MicrosoftYaHei => {
                if has_yahei {
                    prop.insert(0, "msyh".to_owned());
                } else {
                    prop.insert(0, "fzgwkai".to_owned());
                }
            }
        }
    }
    {
        let mono = fonts.families.entry(egui::FontFamily::Monospace).or_default();
        match choice {
            FontChoice::FzGwKai => {
                mono.insert(0, "fzgwkai".to_owned());
                if has_mono {
                    mono.insert(1, "consola".to_owned());
                }
            }
            FontChoice::MicrosoftYaHei => {
                if has_mono {
                    mono.insert(0, "consola".to_owned());
                }
                if has_yahei {
                    mono.insert(usize::from(has_mono), "msyh".to_owned());
                }
            }
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context, p: Palette, m: DensityMetrics) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 6.0);
    style.spacing.interact_size = egui::vec2(40.0, m.control_height);
    style.spacing.scroll = egui::style::ScrollStyle::thin();

    let mut v = if p.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p.bg_canvas;
    v.window_fill = p.bg_surface;
    v.extreme_bg_color = p.bg_control;
    v.faint_bg_color = p.bg_control_alt;
    v.window_stroke = Stroke::new(1.0, p.border_control);
    v.window_corner_radius = CornerRadius::same(8);
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.override_text_color = Some(p.text_primary);

    v.widgets.noninteractive.bg_fill = p.bg_surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border_subtle);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text_primary);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    v.widgets.inactive.bg_fill = p.bg_control;
    v.widgets.inactive.weak_bg_fill = p.bg_control;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border_control);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text_primary);
    v.widgets.inactive.corner_radius = CornerRadius::same(4);

    v.widgets.hovered.bg_fill = p.bg_hover;
    v.widgets.hovered.weak_bg_fill = p.bg_hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.border_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text_primary);
    v.widgets.hovered.corner_radius = CornerRadius::same(4);
    v.widgets.hovered.expansion = 0.0;

    v.widgets.active.bg_fill = p.bg_pressed;
    v.widgets.active.weak_bg_fill = p.bg_pressed;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent_primary);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text_primary);
    v.widgets.active.corner_radius = CornerRadius::same(4);
    v.widgets.active.expansion = 0.0;

    v.widgets.open.bg_fill = p.bg_hover;
    v.widgets.open.bg_stroke = Stroke::new(1.0, p.focus_ring);
    v.widgets.open.fg_stroke = Stroke::new(1.0, p.text_primary);
    v.widgets.open.corner_radius = CornerRadius::same(4);

    // egui 中 selection.stroke.color 为选中文字前景色；采用浅色强调底 + 主文本色，
    // 确保深浅模式下均具备高对比度，并完全规避 egui 0.33.3 折行末字漏改色导致的视觉割裂
    v.selection.bg_fill = p.bg_selected;
    v.selection.stroke = Stroke::new(1.5, p.text_primary);

    style.visuals = v;
    ctx.set_style(style);
}
