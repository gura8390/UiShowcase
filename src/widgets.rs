use crate::theme::{DensityMetrics, Palette};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, WidgetType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Danger,
}

pub fn surface_card<R>(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(p.bg_surface)
        .stroke(Stroke::new(1.0, p.border_subtle))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(m.card_pad))
        .show(ui, |ui| ui.vertical(add_contents).inner)
}

/// 精确视觉居中按钮（带五态、2px 键盘焦点外描边、AccessKit 中文无障碍名称）
pub fn styled_button(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    label: &str,
    kind: ButtonKind,
    enabled: bool,
    full_width: bool,
) -> Response {
    let font_id = FontId::proportional(m.body_size);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_id.clone(), Color32::WHITE);
    let pad_x = if matches!(kind, ButtonKind::Primary) {
        18.0
    } else {
        14.0
    };
    let min_w = (galley.size().x + pad_x * 2.0).max(68.0);
    let width = if full_width {
        ui.available_width().max(48.0)
    } else {
        min_w
    };
    let desired = Vec2::new(width, m.control_height);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(desired, sense);

    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let focused = enabled && response.has_focus();

        let (bg, border, fg) = if !enabled {
            (p.bg_disabled, p.border_subtle, p.text_disabled)
        } else {
            match kind {
                ButtonKind::Primary => {
                    let bg = if pressed {
                        p.accent_pressed
                    } else if hovered {
                        p.accent_hover
                    } else {
                        p.accent_primary
                    };
                    (bg, bg, p.text_on_accent)
                }
                ButtonKind::Secondary => {
                    let bg = if pressed {
                        p.bg_pressed
                    } else if hovered {
                        p.bg_hover
                    } else {
                        p.bg_control
                    };
                    let bd = if hovered || pressed {
                        p.border_strong
                    } else {
                        p.border_control
                    };
                    (bg, bd, p.text_primary)
                }
                ButtonKind::Ghost => {
                    let bg = if pressed {
                        p.bg_pressed
                    } else if hovered {
                        p.bg_hover
                    } else {
                        Color32::TRANSPARENT
                    };
                    (bg, Color32::TRANSPARENT, p.text_primary)
                }
                ButtonKind::Danger => {
                    let bg = if pressed || hovered {
                        p.status_danger_hover
                    } else {
                        p.status_danger_btn
                    };
                    (bg, bg, Color32::WHITE)
                }
            }
        };

        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(4),
            bg,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );

        if focused {
            painter.rect_stroke(
                rect.expand(2.5),
                CornerRadius::same(6),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        // Align2::CENTER_CENTER 直接按字形包围盒居中，彻底消除中文字形上下偏斜
        painter.text(rect.center(), Align2::CENTER_CENTER, label, font_id, fg);
    }

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// 左侧导航项（完整五态 + 可选左侧 3px 选中指示条 + 窄栏居中精简排版 + AccessKit 完整中文语义）
#[allow(clippy::too_many_arguments)]
pub fn nav_item(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    display_label: &str,
    accessible_label: &str,
    selected: bool,
    compact: bool,
    draw_indicator: bool,
) -> Response {
    let enabled = ui.is_enabled();
    let desired = Vec2::new(ui.available_width(), m.nav_item_height);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            enabled,
            selected,
            format!("导航：{accessible_label}"),
        )
    });

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let focused = enabled && response.has_focus();

        let (bg, fg) = if !enabled {
            (
                if selected { p.bg_disabled } else { Color32::TRANSPARENT },
                p.text_disabled,
            )
        } else if pressed {
            (p.bg_pressed, p.text_primary)
        } else if selected {
            (p.bg_selected, p.text_primary)
        } else if hovered {
            (p.bg_hover, p.text_primary)
        } else {
            (Color32::TRANSPARENT, p.text_secondary)
        };

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), bg);

        if selected && draw_indicator {
            let ind_h = 18.0;
            let ind_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + 4.0, rect.center().y - ind_h / 2.0),
                Vec2::new(3.0, ind_h),
            );
            painter.rect_filled(
                ind_rect,
                CornerRadius::same(2),
                if enabled { p.accent_primary } else { p.text_disabled },
            );
        }

        if focused {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(7),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        let font_size = if compact && rect.width() < 84.0 {
            m.secondary_size
        } else {
            m.body_size
        };
        if compact {
            painter.text(
                egui::pos2(rect.center().x + 2.0, rect.center().y),
                Align2::CENTER_CENTER,
                display_label,
                FontId::proportional(font_size),
                fg,
            );
        } else {
            painter.text(
                egui::pos2(rect.left() + 18.0, rect.center().y),
                Align2::LEFT_CENTER,
                display_label,
                FontId::proportional(font_size),
                fg,
            );
        }
    }

    let resp = if compact {
        response.on_hover_text(accessible_label)
    } else {
        response
    };
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// 分段选择控件（完整五态 + 水平与垂直双居中 + AccessKit 语义）
pub fn segment_bar<T: Copy + PartialEq>(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    current: &mut T,
    options: &[(T, &str)],
    group_prefix: &str,
) -> bool {
    let mut changed = false;
    let enabled = ui.is_enabled();
    egui::Frame::new()
        .fill(if enabled { p.bg_control_alt } else { p.bg_disabled })
        .stroke(Stroke::new(1.0, p.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::same(2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for &(val, label) in options {
                    let selected = *current == val;
                    let font_id = FontId::proportional(m.secondary_size);
                    let galley = ui.painter().layout_no_wrap(
                        label.to_owned(),
                        font_id.clone(),
                        p.text_primary,
                    );
                    let size = Vec2::new((galley.size().x + 22.0).max(52.0), m.control_height - 6.0);
                    let sense = if enabled { Sense::click() } else { Sense::hover() };
                    let (rect, resp) = ui.allocate_exact_size(size, sense);
                    resp.widget_info(|| {
                        WidgetInfo::selected(
                            WidgetType::RadioButton,
                            enabled,
                            selected,
                            format!("{group_prefix}：{label}"),
                        )
                    });

                    if ui.is_rect_visible(rect) {
                        let hovered = enabled && resp.hovered();
                        let pressed = enabled && resp.is_pointer_button_down_on();
                        let focused = enabled && resp.has_focus();

                        let painter = ui.painter();
                        let (bg, stroke, fg) = if !enabled {
                            if selected {
                                (
                                    p.bg_surface,
                                    Stroke::new(1.0, p.border_subtle),
                                    p.text_disabled,
                                )
                            } else {
                                (Color32::TRANSPARENT, Stroke::NONE, p.text_disabled)
                            }
                        } else if pressed {
                            (
                                p.bg_pressed,
                                Stroke::new(1.0, p.border_strong),
                                p.text_primary,
                            )
                        } else if selected {
                            (
                                p.bg_surface,
                                Stroke::new(1.0, p.border_control),
                                p.text_primary,
                            )
                        } else if hovered {
                            (p.bg_hover, Stroke::NONE, p.text_primary)
                        } else {
                            (Color32::TRANSPARENT, Stroke::NONE, p.text_secondary)
                        };
                        painter.rect(rect, CornerRadius::same(4), bg, stroke, StrokeKind::Inside);
                        if focused {
                            painter.rect_stroke(
                                rect.expand(2.0),
                                CornerRadius::same(5),
                                Stroke::new(2.0, p.focus_ring),
                                StrokeKind::Outside,
                            );
                        }
                        painter.text(rect.center(), Align2::CENTER_CENTER, label, font_id, fg);
                    }

                    let resp = if enabled {
                        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
                    } else {
                        resp
                    };
                    if enabled && resp.clicked() && !selected {
                        *current = val;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// 现代拨动开关（完整五态 + 100ms 顺滑圆点动效且尊重系统减少动画 + 窄窗自动换行防溢出 + AccessKit 语义）
pub fn toggle_switch(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    checked: &mut bool,
    label: &str,
) -> Response {
    let enabled = ui.is_enabled();
    let font_id = FontId::proportional(m.body_size);
    let track_w: f32 = 40.0;
    let track_h: f32 = 22.0;
    let gap = 10.0;
    let max_label_w = (ui.available_width() - track_w - gap - 4.0).max(80.0);
    let label_color = if enabled { p.text_primary } else { p.text_disabled };
    let galley = layout_left_wrapped(ui.painter(), label, font_id, label_color, max_label_w);
    let total_w = (track_w + gap + galley.size().x + 4.0).min(ui.available_width().max(track_w + gap + 80.0));
    let total_h = track_h.max(galley.size().y + 4.0);

    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_w, total_h), sense);
    if enabled && response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }

    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, *checked, label));

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let focused = enabled && response.has_focus();

        let painter = ui.painter();
        let track_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - track_h / 2.0),
            Vec2::new(track_w, track_h),
        );
        let (track_bg, track_border, thumb_fill) = if !enabled {
            (p.bg_disabled, p.border_subtle, p.text_disabled)
        } else if *checked {
            let bg = if pressed {
                p.accent_pressed
            } else if hovered {
                p.accent_hover
            } else {
                p.accent_primary
            };
            (bg, bg, p.text_on_accent)
        } else {
            let bg = if pressed {
                p.bg_pressed
            } else if hovered {
                p.bg_hover
            } else {
                p.bg_control_alt
            };
            (bg, p.border_strong, p.text_secondary)
        };

        painter.rect(
            track_rect,
            CornerRadius::same(11),
            track_bg,
            Stroke::new(1.0, track_border),
            StrokeKind::Inside,
        );

        let anim_secs = if crate::system::prefers_reduced_motion() {
            0.0
        } else {
            0.10
        };
        let t = ui.ctx().animate_bool_with_time(response.id, *checked, anim_secs);
        let thumb_r = if pressed { 7.5 } else { 7.0 };
        let x_off = track_rect.left() + 4.0 + 7.0;
        let x_on = track_rect.right() - 4.0 - 7.0;
        let thumb_x = egui::lerp(x_off..=x_on, t);
        painter.circle_filled(egui::pos2(thumb_x, track_rect.center().y), thumb_r, thumb_fill);

        if focused {
            painter.rect_stroke(
                track_rect.expand(2.5),
                CornerRadius::same(13),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        let text_y = (rect.center().y - galley.size().y * 0.5).round();
        painter.galley(
            egui::pos2(track_rect.right() + gap, text_y),
            galley,
            label_color,
        );
    }

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// 语义状态徽章（视觉居中胶囊；非交互展示标签允许拖选复制）
pub fn status_badge(ui: &mut Ui, p: Palette, m: DensityMetrics, text: &str, kind: StatusKind) {
    let (fg, bg, bd) = status_colors(p, kind);
    let font_id = FontId::proportional(m.caption_size);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_id, fg);
    let size = Vec2::new(galley.size().x + 18.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            CornerRadius::same(4),
            bg,
            Stroke::new(1.0, bd),
            StrokeKind::Inside,
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                )),
        );
        child.add(
            egui::Label::new(egui::RichText::new(text).size(m.caption_size).color(fg))
                .selectable(true),
        );
    }
}

/// 语义提示条（四态反馈：带文字标记前缀 + 语义色；强制纵向排版避免受父级 horizontal 污染）
pub fn status_banner(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    kind: StatusKind,
    title: &str,
    desc: &str,
) {
    let (fg, bg, bd) = status_colors(p, kind);
    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, bd))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(title).size(m.body_size).color(fg));
                if !desc.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(desc)
                            .size(m.secondary_size)
                            .color(p.text_primary),
                    );
                }
            });
        });
}

pub fn status_colors(p: Palette, kind: StatusKind) -> (Color32, Color32, Color32) {
    match kind {
        StatusKind::Info => (p.status_info, p.status_info_bg, p.status_info_border),
        StatusKind::Success => (
            p.status_success,
            p.status_success_bg,
            p.status_success_border,
        ),
        StatusKind::Warning => (
            p.status_warning,
            p.status_warning_bg,
            p.status_warning_border,
        ),
        StatusKind::Danger => (p.status_danger, p.status_danger_bg, p.status_danger_border),
    }
}

struct WorkbenchCols {
    id_left: f32,
    name_left: f32,
    name_width: f32,
    size_right: f32,
    status_center_x: f32,
    status_width: f32,
}

fn workbench_cols(rect: egui::Rect) -> WorkbenchCols {
    let pad_l = 16.0;
    let pad_r = 12.0;
    let id_w = 74.0;
    let size_w = 82.0;
    let status_w = 82.0;
    let gap = 10.0;

    let id_left = rect.left() + pad_l;
    let status_right = rect.right() - pad_r;
    let status_left = status_right - status_w;
    let size_right = status_left - gap;
    let name_left = id_left + id_w + gap;
    let name_width = (size_right - size_w - gap - name_left).max(96.0);

    WorkbenchCols {
        id_left,
        name_left,
        name_width,
        size_right,
        status_center_x: (status_left + status_right) * 0.5,
        status_width: status_w,
    }
}

/// 核查记录列表四列表头（编号 / 文件名称与位置 / 大小 / 状态；字号高于正文且允许拖选复制）
pub fn workbench_task_header(ui: &mut Ui, p: Palette, m: DensityMetrics, width: f32) {
    let header_h = if m.control_height <= 30.0 { 32.0 } else { 38.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, header_h), Sense::hover());
    if ui.is_rect_visible(rect) {
        let cols = workbench_cols(rect);
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(5), p.bg_canvas);
        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, p.border_control),
        );

        let place_hdr = |ui: &mut Ui, col_rect: egui::Rect, layout: egui::Layout, text: &str| {
            let mut child =
                ui.new_child(egui::UiBuilder::new().max_rect(col_rect).layout(layout));
            child.add(
                egui::Label::new(
                    egui::RichText::new(text)
                        .size(m.h3_size)
                        .color(p.text_primary),
                )
                .selectable(true),
            );
        };

        let id_rect = egui::Rect::from_min_max(
            egui::pos2(cols.id_left, rect.top()),
            egui::pos2(cols.name_left - 4.0, rect.bottom()),
        );
        let name_rect = egui::Rect::from_min_max(
            egui::pos2(cols.name_left, rect.top()),
            egui::pos2(cols.name_left + cols.name_width, rect.bottom()),
        );
        let size_rect = egui::Rect::from_min_max(
            egui::pos2(cols.size_right - 82.0, rect.top()),
            egui::pos2(cols.size_right, rect.bottom()),
        );
        let status_rect = egui::Rect::from_min_max(
            egui::pos2(cols.status_center_x - cols.status_width * 0.5, rect.top()),
            egui::pos2(cols.status_center_x + cols.status_width * 0.5, rect.bottom()),
        );

        place_hdr(
            ui,
            id_rect,
            egui::Layout::left_to_right(egui::Align::Center),
            "编号",
        );
        place_hdr(
            ui,
            name_rect,
            egui::Layout::left_to_right(egui::Align::Center),
            "文件名称与位置",
        );
        place_hdr(
            ui,
            size_rect,
            egui::Layout::right_to_left(egui::Align::Center),
            "大小",
        );
        place_hdr(
            ui,
            status_rect,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            "状态",
        );
    }
}

fn layout_ellipsized(
    painter: &egui::Painter,
    text: &str,
    font_id: FontId,
    color: Color32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width: max_width.max(40.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    painter.layout_job(job)
}

/// 核查记录列表严格四列单行控件（裁剪省略不重叠 + 五态反馈 + 左侧选中指示条 + AccessKit 语义）
#[allow(clippy::too_many_arguments)]
pub fn workbench_task_row(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    width: f32,
    id: &str,
    file_name: &str,
    folder: &str,
    size: &str,
    status: &str,
    kind: StatusKind,
    selected: bool,
) -> Response {
    let enabled = ui.is_enabled();
    let row_h = if m.control_height <= 30.0 { 48.0 } else { 58.0 };
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, row_h), sense);

    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            enabled,
            selected,
            format!("{id} {file_name} {status}"),
        )
    });

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let focused = enabled && response.has_focus();
        let bg = if !enabled {
            p.bg_disabled
        } else if pressed {
            p.bg_pressed
        } else if selected {
            p.bg_selected
        } else if hovered {
            p.bg_hover
        } else {
            Color32::TRANSPARENT
        };
        let border = if !enabled {
            p.border_subtle
        } else if selected {
            p.accent_primary
        } else if pressed || hovered {
            p.border_control
        } else {
            p.border_subtle
        };

        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(6),
            bg,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );

        if selected {
            let ind_h = (row_h * 0.48).round();
            let ind_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + 4.0, rect.center().y - ind_h * 0.5),
                Vec2::new(3.0, ind_h),
            );
            painter.rect_filled(
                ind_rect,
                CornerRadius::same(2),
                if enabled { p.accent_primary } else { p.text_disabled },
            );
        }

        if focused {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(7),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        let cols = workbench_cols(rect);
        let cy = rect.center().y;

        let id_color = if enabled { p.text_secondary } else { p.text_disabled };
        let name_color = if enabled { p.text_primary } else { p.text_disabled };
        let folder_color = if enabled { p.text_muted } else { p.text_disabled };

        // 列 0：编号
        painter.text(
            egui::pos2(cols.id_left, cy),
            Align2::LEFT_CENTER,
            id,
            FontId::monospace(m.caption_size),
            id_color,
        );

        // 列 1：文件名 + 目录（严格裁剪与单行省略，绝不越界重叠）
        let name_galley = layout_ellipsized(
            painter,
            file_name,
            FontId::proportional(m.body_size),
            name_color,
            cols.name_width,
        );
        let folder_galley = layout_ellipsized(
            painter,
            folder,
            FontId::proportional(m.caption_size),
            folder_color,
            cols.name_width,
        );
        let line_gap = 3.0;
        let total_text_h = name_galley.size().y + line_gap + folder_galley.size().y;
        let top_y = cy - total_text_h * 0.5;
        let name_clip = egui::Rect::from_min_size(
            egui::pos2(cols.name_left, rect.top() + 2.0),
            Vec2::new(cols.name_width + 2.0, rect.height() - 4.0),
        );
        let name_painter = painter.with_clip_rect(name_clip);
        name_painter.galley(egui::pos2(cols.name_left, top_y), name_galley.clone(), name_color);
        name_painter.galley(
            egui::pos2(cols.name_left, top_y + name_galley.size().y + line_gap),
            folder_galley,
            folder_color,
        );

        // 列 2：大小（右对齐等宽）
        painter.text(
            egui::pos2(cols.size_right, cy),
            Align2::RIGHT_CENTER,
            size.trim(),
            FontId::monospace(m.caption_size),
            id_color,
        );

        // 列 3：状态胶囊徽章（几何居中）
        let (fg, badge_bg, badge_bd) = if enabled {
            status_colors(p, kind)
        } else {
            (p.text_disabled, p.bg_disabled, p.border_subtle)
        };
        let badge_font = FontId::proportional(m.caption_size);
        let badge_galley = painter.layout_no_wrap(status.to_owned(), badge_font.clone(), fg);
        let badge_w = (badge_galley.size().x + 18.0).min(cols.status_width);
        let badge_rect = egui::Rect::from_center_size(
            egui::pos2(cols.status_center_x, cy),
            Vec2::new(badge_w, 24.0),
        );
        painter.rect(
            badge_rect,
            CornerRadius::same(4),
            badge_bg,
            Stroke::new(1.0, badge_bd),
            StrokeKind::Inside,
        );
        painter.text(badge_rect.center(), Align2::CENTER_CENTER, status, badge_font, fg);
    }

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

fn layout_left_wrapped(
    painter: &egui::Painter,
    text: &str,
    font_id: FontId,
    color: Color32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );
    job.halign = egui::Align::LEFT;
    job.wrap = egui::text::TextWrapping {
        max_width: max_width.max(80.0),
        ..Default::default()
    };
    painter.layout_job(job)
}

/// 等宽等高且版式统一的核心原则卡片行：
/// - 四张卡片按最高卡片锁定完全一致的宽高；
/// - 标题与多行说明统一严格左对齐（避免居中多行换行导致行首缩进参差）；
/// - 整组文字块在卡片内部垂直视觉居中，左右内边距对称。
pub fn principle_card_row(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    cards: &[(&str, &str)],
) {
    if cards.is_empty() {
        return;
    }
    let gap = 12.0;
    let avail_w = ui.available_width();
    // 窄窗（如 820px 最小窗口下可用宽 ~540pt）自动从 4×1 折为 2×2 等高网格，杜绝横向溢出
    let cols = if avail_w >= 700.0 {
        cards.len().min(4)
    } else if avail_w >= 360.0 {
        2
    } else {
        1
    };
    let card_w = ((avail_w - gap * (cols as f32 - 1.0)) / cols as f32)
        .floor()
        .max(150.0);
    let pad_x = f32::from(m.card_pad).max(16.0);
    let pad_y = f32::from(m.card_pad).max(14.0);
    let text_w = (card_w - pad_x * 2.0).max(100.0);
    let title_gap = 8.0;

    // 先统一预排全部卡片字形，按最高卡片统一锁定等高尺寸
    let painter = ui.painter();
    let layouts: Vec<_> = cards
        .iter()
        .map(|&(title, desc)| {
            let tg = layout_left_wrapped(
                painter,
                title,
                FontId::proportional(m.h3_size),
                p.text_primary,
                text_w,
            );
            let dg = layout_left_wrapped(
                painter,
                desc,
                FontId::proportional(m.secondary_size),
                p.text_secondary,
                text_w,
            );
            (tg, dg)
        })
        .collect();

    let max_content_h = layouts
        .iter()
        .map(|(tg, dg)| tg.size().y + title_gap + dg.size().y)
        .fold(0.0_f32, f32::max);
    let card_h = (max_content_h + pad_y * 2.0).max(116.0).ceil();

    for (row_idx, row_start) in (0..cards.len()).step_by(cols).enumerate() {
        if row_idx > 0 {
            ui.add_space(gap);
        }
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for offset in 0..cols {
                let idx = row_start + offset;
                if idx >= cards.len() {
                    break;
                }
                let (title, desc) = cards[idx];
                let (ref tg, ref dg) = layouts[idx];
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(card_w, card_h), Sense::hover());
                resp.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Panel, true, format!("原则卡片：{title}"))
                });

                if ui.is_rect_visible(rect) {
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(8),
                        p.bg_surface,
                        Stroke::new(1.0, p.border_subtle),
                        StrokeKind::Inside,
                    );

                    let block_h = tg.size().y + title_gap + dg.size().y;
                    let top_y = (rect.center().y - block_h * 0.5).round();
                    let left_x = (rect.left() + pad_x).round();
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(left_x, top_y),
                        Vec2::new(text_w, (rect.bottom() - top_y - 4.0).max(block_h)),
                    );
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(text_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    child.set_width(text_w);
                    child.set_max_width(text_w);
                    child.spacing_mut().item_spacing.y = title_gap;
                    child.add(
                        egui::Label::new(
                            egui::RichText::new(title)
                                .size(m.h3_size)
                                .color(p.text_primary),
                        )
                        .selectable(true)
                        .wrap(),
                    );
                    child.add(
                        egui::Label::new(
                            egui::RichText::new(desc)
                                .size(m.secondary_size)
                                .color(p.text_secondary),
                        )
                        .selectable(true)
                        .wrap(),
                    );
                }
            }
        });
    }
}

/// 自绘现代下拉选择控件（等高对齐按钮、抗锯齿矢量箭头、深浅色自适应弹层、AccessKit 完整语义）
pub fn styled_dropdown(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    id_salt: &str,
    accessible_label: &str,
    current: &mut &'static str,
    options: &[&'static str],
    width: f32,
) -> bool {
    let mut changed = false;
    let enabled = ui.is_enabled();
    let popup_id = ui.make_persistent_id(id_salt);
    let is_open = enabled
        && ui
            .ctx()
            .memory(|mem| mem.data.get_temp::<bool>(popup_id))
            .unwrap_or(false);

    let desired = Vec2::new(width.max(96.0), m.control_height);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(desired, sense);
    response.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::ComboBox,
            enabled,
            format!("{accessible_label}：{}", *current),
        )
    });

    if enabled && response.clicked() {
        ui.ctx()
            .memory_mut(|mem| mem.data.insert_temp(popup_id, !is_open));
    }

    let now_open = enabled
        && ui
            .ctx()
            .memory(|mem| mem.data.get_temp::<bool>(popup_id))
            .unwrap_or(false);

    if ui.is_rect_visible(rect) {
        let hovered = enabled && response.hovered();
        let pressed = enabled && response.is_pointer_button_down_on();
        let focused = enabled && response.has_focus();

        let (bg, border, fg, arrow_color) = if !enabled {
            (p.bg_disabled, p.border_subtle, p.text_disabled, p.text_disabled)
        } else {
            let bg = if pressed {
                p.bg_pressed
            } else if hovered || now_open {
                p.bg_hover
            } else {
                p.bg_control
            };
            let border = if now_open {
                p.accent_primary
            } else if hovered || pressed {
                p.border_strong
            } else {
                p.border_control
            };
            let arrow = if now_open {
                p.accent_primary
            } else {
                p.text_secondary
            };
            (bg, border, p.text_primary, arrow)
        };

        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(4),
            bg,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );

        if focused {
            painter.rect_stroke(
                rect.expand(2.5),
                CornerRadius::same(6),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        // 左侧当前选项文字（严格裁剪不碰右侧箭头）
        let text_left = rect.left() + 12.0;
        let chevron_area_w = 28.0;
        let text_max_w = (rect.width() - 12.0 - chevron_area_w).max(32.0);
        let galley = layout_ellipsized(
            painter,
            current,
            FontId::proportional(m.body_size),
            fg,
            text_max_w,
        );
        let text_y = rect.center().y - galley.size().y * 0.5;
        let text_clip = egui::Rect::from_min_size(
            egui::pos2(text_left, rect.top() + 2.0),
            Vec2::new(text_max_w, rect.height() - 4.0),
        );
        painter
            .with_clip_rect(text_clip)
            .galley(egui::pos2(text_left, text_y), galley, fg);

        // 右侧精致抗锯齿矢量折线箭头（展开时翻转朝上）
        let cx = rect.right() - 15.0;
        let cy = rect.center().y;
        let arrow_stroke = Stroke::new(1.5, arrow_color);
        if now_open {
            painter.line_segment([egui::pos2(cx - 4.0, cy + 2.0), egui::pos2(cx, cy - 2.0)], arrow_stroke);
            painter.line_segment([egui::pos2(cx, cy - 2.0), egui::pos2(cx + 4.0, cy + 2.0)], arrow_stroke);
        } else {
            painter.line_segment([egui::pos2(cx - 4.0, cy - 2.0), egui::pos2(cx, cy + 2.0)], arrow_stroke);
            painter.line_segment([egui::pos2(cx, cy + 2.0), egui::pos2(cx + 4.0, cy - 2.0)], arrow_stroke);
        }
    }

    if enabled {
        let _ = response.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    if now_open {
        let mut close_popup = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
        let popup_width = width.max(128.0);
        let area_resp = egui::Area::new(popup_id.with("popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(rect.left(), rect.bottom() + 4.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(p.bg_surface)
                    .stroke(Stroke::new(1.0, p.border_control))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(4))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            let row_w = popup_width - 8.0;
                            let row_h = (m.control_height - 2.0).max(28.0);
                            for &opt in options {
                                let is_sel = *current == opt;
                                let (r, resp) = ui.allocate_exact_size(
                                    Vec2::new(row_w, row_h),
                                    Sense::click(),
                                );
                                resp.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        true,
                                        is_sel,
                                        opt,
                                    )
                                });

                                if ui.is_rect_visible(r) {
                                    let pnt = ui.painter();
                                    let row_bg = if resp.is_pointer_button_down_on() {
                                        p.bg_pressed
                                    } else if is_sel {
                                        p.bg_selected
                                    } else if resp.hovered() {
                                        p.bg_hover
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    pnt.rect_filled(r, CornerRadius::same(4), row_bg);
                                    if resp.has_focus() {
                                        pnt.rect_stroke(
                                            r.expand(1.5),
                                            CornerRadius::same(5),
                                            Stroke::new(2.0, p.focus_ring),
                                            StrokeKind::Outside,
                                        );
                                    }
                                    if is_sel {
                                        let ind_h = 14.0;
                                        let ind_rect = egui::Rect::from_min_size(
                                            egui::pos2(r.left() + 4.0, r.center().y - ind_h * 0.5),
                                            Vec2::new(3.0, ind_h),
                                        );
                                        pnt.rect_filled(
                                            ind_rect,
                                            CornerRadius::same(2),
                                            p.accent_primary,
                                        );
                                    }
                                    pnt.text(
                                        egui::pos2(r.left() + 14.0, r.center().y),
                                        Align2::LEFT_CENTER,
                                        opt,
                                        FontId::proportional(m.body_size),
                                        if is_sel { p.accent_primary } else { p.text_primary },
                                    );
                                }

                                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    if *current != opt {
                                        *current = opt;
                                        changed = true;
                                    }
                                    close_popup = true;
                                }
                            }
                        });
                    });
            });

        // 点击弹层与触发按钮以外区域时自动收起
        if ui.ctx().input(|i| i.pointer.any_click()) {
            let clicked_inside = ui.ctx().pointer_interact_pos().is_some_and(|pos| {
                rect.contains(pos) || area_resp.response.rect.contains(pos)
            });
            if !clicked_inside {
                close_popup = true;
            }
        }

        if close_popup {
            ui.ctx()
                .memory_mut(|mem| mem.data.insert_temp(popup_id, false));
        }
    }

    changed
}

/// 自绘高对比度滑动条（完整五态 + 明确底轨 + 强调色已选导轨 + 圆形滑钮 + 键盘微调 + AccessKit Slider 语义）
pub fn styled_slider(
    ui: &mut Ui,
    p: Palette,
    value: &mut f32,
    min: f32,
    max: f32,
    step: f32,
    width: f32,
    accessible_label: &str,
) -> Response {
    let enabled = ui.is_enabled();
    let height = 28.0;
    let thumb_r = 8.5_f32;
    let sense = if enabled {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width.max(120.0), height), sense);

    let rail_left = rect.left() + thumb_r + 2.0;
    let rail_right = (rect.right() - thumb_r - 2.0).max(rail_left + 1.0);
    let rail_w = rail_right - rail_left;

    if enabled
        && (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - rail_left) / rail_w).clamp(0.0, 1.0);
        let raw = min + t * (max - min);
        let snapped = if step > 0.0 {
            ((raw - min) / step).round() * step + min
        } else {
            raw
        }
        .clamp(min, max);
        if (snapped - *value).abs() > f32::EPSILON {
            *value = snapped;
            response.mark_changed();
        }
    }

    if enabled && response.has_focus() {
        let delta = ui.ctx().input(|i| {
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp) {
                step
            } else if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) {
                -step
            } else {
                0.0
            }
        });
        if delta != 0.0 {
            let next = (*value + delta).clamp(min, max);
            if (next - *value).abs() > f32::EPSILON {
                *value = next;
                response.mark_changed();
            }
        }
    }

    response.widget_info(|| WidgetInfo::slider(enabled, f64::from(*value), accessible_label));

    if ui.is_rect_visible(rect) {
        let t = ((*value - min) / (max - min).max(f32::EPSILON)).clamp(0.0, 1.0);
        let cy = rect.center().y;
        let thumb_x = rail_left + t * rail_w;
        let rail_h = 6.0;
        let rail_rect = egui::Rect::from_min_max(
            egui::pos2(rail_left, cy - rail_h * 0.5),
            egui::pos2(rail_right, cy + rail_h * 0.5),
        );
        let active_rect = egui::Rect::from_min_max(
            egui::pos2(rail_left, cy - rail_h * 0.5),
            egui::pos2(thumb_x.max(rail_left + 3.0), cy + rail_h * 0.5),
        );

        let hovered = enabled && response.hovered();
        let pressed = enabled && (response.dragged() || response.is_pointer_button_down_on());
        let focused = enabled && response.has_focus();

        let (rail_fill, rail_border, active_fill, thumb_ring) = if !enabled {
            (p.bg_disabled, p.border_subtle, p.border_control, p.text_disabled)
        } else if pressed {
            (p.bg_control_alt, p.border_strong, p.accent_pressed, p.accent_pressed)
        } else if hovered {
            (p.bg_control_alt, p.border_strong, p.accent_hover, p.accent_hover)
        } else {
            (p.bg_control_alt, p.border_control, p.accent_primary, p.accent_primary)
        };

        let painter = ui.painter();
        painter.rect(
            rail_rect,
            CornerRadius::same(3),
            rail_fill,
            Stroke::new(1.0, rail_border),
            StrokeKind::Inside,
        );
        painter.rect_filled(active_rect, CornerRadius::same(3), active_fill);

        let draw_r = if pressed || hovered { thumb_r + 1.0 } else { thumb_r };
        let thumb_center = egui::pos2(thumb_x, cy);
        if focused {
            painter.circle_stroke(thumb_center, draw_r + 3.0, Stroke::new(2.0, p.focus_ring));
        }
        painter.circle(
            thumb_center,
            draw_r,
            if enabled { p.bg_surface } else { p.bg_disabled },
            Stroke::new(2.2, thumb_ring),
        );
    }

    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

/// 等宽等高的状态反馈横幅网格（宽屏 2×2，窄窗自动降为 1 列，防溢出且尺寸完全一致）
pub fn status_banner_grid(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    items: &[(StatusKind, &str, &str)],
    total_width: f32,
) {
    if items.is_empty() {
        return;
    }
    let gap_x = 12.0;
    let gap_y = 10.0;
    let cols = if total_width >= 460.0 { 2 } else { 1 };
    let bw = if cols == 2 {
        ((total_width - gap_x) * 0.5).floor().max(180.0)
    } else {
        total_width.floor().max(180.0)
    };
    let pad_x = 14.0;
    let pad_y = 12.0;
    let title_gap = 5.0;
    let text_w = (bw - pad_x * 2.0).max(120.0);

    let painter = ui.painter();
    let layouts: Vec<_> = items
        .iter()
        .map(|&(kind, title, desc)| {
            let (fg, _, _) = status_colors(p, kind);
            let tg = layout_left_wrapped(
                painter,
                title,
                FontId::proportional(m.body_size),
                fg,
                text_w,
            );
            let dg = layout_left_wrapped(
                painter,
                desc,
                FontId::proportional(m.secondary_size),
                p.text_primary,
                text_w,
            );
            (tg, dg)
        })
        .collect();

    let max_content_h = layouts
        .iter()
        .map(|(tg, dg)| tg.size().y + title_gap + dg.size().y)
        .fold(0.0_f32, f32::max);
    let box_h = (max_content_h + pad_y * 2.0).max(74.0).ceil();

    for row_idx in (0..items.len()).step_by(cols) {
        if row_idx > 0 {
            ui.add_space(gap_y);
        }
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap_x;
            for offset in 0..cols {
                let idx = row_idx + offset;
                if idx >= items.len() {
                    break;
                }
                let (kind, title, desc) = items[idx];
                let (ref tg, ref dg) = layouts[idx];
                let (fg, bg, bd) = status_colors(p, kind);

                let (rect, resp) = ui.allocate_exact_size(Vec2::new(bw, box_h), Sense::hover());
                resp.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Panel, true, format!("状态横幅：{title}"))
                });

                if ui.is_rect_visible(rect) {
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(6),
                        bg,
                        Stroke::new(1.0, bd),
                        StrokeKind::Inside,
                    );
                    let block_h = tg.size().y + title_gap + dg.size().y;
                    let top_y = (rect.center().y - block_h * 0.5).round();
                    let left_x = (rect.left() + pad_x).round();
                    let text_rect = egui::Rect::from_min_size(
                        egui::pos2(left_x, top_y),
                        Vec2::new(text_w, (rect.bottom() - top_y - 4.0).max(block_h)),
                    );
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(text_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    child.set_width(text_w);
                    child.set_max_width(text_w);
                    child.spacing_mut().item_spacing.y = title_gap;
                    child.add(
                        egui::Label::new(
                            egui::RichText::new(title).size(m.body_size).color(fg),
                        )
                        .selectable(true)
                        .wrap(),
                    );
                    child.add(
                        egui::Label::new(
                            egui::RichText::new(desc)
                                .size(m.secondary_size)
                                .color(p.text_primary),
                        )
                        .selectable(true)
                        .wrap(),
                    );
                }
            }
        });
    }
}

#[derive(Clone, Copy, Default)]
struct PairHeightCache {
    target_h: f32,
    left_natural: f32,
    right_natural: f32,
}

/// 同排双栏等高卡片容器：自动测算左右两张卡片的自然内容高度并同步至最大值，
/// 确保左右卡片的顶部与底部边框在任何字体、密度、筛选结果与窗口宽度下严格平齐。
/// 若右侧传入底部固定操作区（如详情页底部的危险操作按钮），留白自动填充在正文与底栏之间，使底栏贴底对齐。
#[allow(clippy::too_many_arguments)]
pub fn equal_height_card_pair(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    id_salt: &str,
    left_w: f32,
    right_w: f32,
    gap: f32,
    add_left: impl FnOnce(&mut Ui),
    add_right_body: impl FnOnce(&mut Ui),
    add_right_footer: Option<impl FnOnce(&mut Ui)>,
) {
    let cache_id = ui.make_persistent_id(("equal_pair_h", id_salt));
    let cache = ui
        .ctx()
        .memory(|mem| mem.data.get_temp::<PairHeightCache>(cache_id))
        .unwrap_or_default();

    let pad = f32::from(m.card_pad);
    let avail_w = ui.available_width();

    // 窄窗降级：当剩余宽度不足以并排容纳双栏时自动改为纵向全宽堆叠，杜绝 820×540 最小窗口溢出
    if avail_w + 1.0 < left_w + right_w + gap {
        let full_inner = (avail_w - pad * 2.0).max(180.0);
        surface_card(ui, p, m, |ui| {
            ui.set_width(full_inner);
            ui.set_max_width(full_inner);
            add_left(ui);
        });
        ui.add_space(m.section_gap);
        surface_card(ui, p, m, |ui| {
            ui.set_width(full_inner);
            ui.set_max_width(full_inner);
            add_right_body(ui);
            if let Some(footer_fn) = add_right_footer {
                footer_fn(ui);
            }
        });
        return;
    }

    let left_inner_w = (left_w - pad * 2.0).max(160.0);
    let right_inner_w = (right_w - pad * 2.0).max(160.0);

    let mut left_natural = 0.0_f32;
    let mut right_natural = 0.0_f32;

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;

        // 左侧卡片
        ui.allocate_ui_with_layout(
            Vec2::new(left_w, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_width(left_w);
                ui.set_max_width(left_w);
                surface_card(ui, p, m, |ui| {
                    ui.set_width(left_inner_w);
                    ui.set_max_width(left_inner_w);
                    let body = ui.vertical(|ui| {
                        ui.set_width(left_inner_w);
                        ui.set_max_width(left_inner_w);
                        add_left(ui);
                    });
                    left_natural = body.response.rect.height();
                    let extra = (cache.target_h - left_natural).max(0.0);
                    if extra > 0.5 {
                        ui.add_space(extra);
                    }
                });
            },
        );

        // 右侧卡片（可选贴底操作栏）
        ui.allocate_ui_with_layout(
            Vec2::new(right_w, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_width(right_w);
                ui.set_max_width(right_w);
                surface_card(ui, p, m, |ui| {
                    ui.set_width(right_inner_w);
                    ui.set_max_width(right_inner_w);
                    let body = ui.vertical(|ui| {
                        ui.set_width(right_inner_w);
                        ui.set_max_width(right_inner_w);
                        add_right_body(ui);
                    });
                    let body_h = body.response.rect.height();

                    if let Some(footer_fn) = add_right_footer {
                        let extra = (cache.target_h - cache.right_natural).max(0.0);
                        if extra > 0.5 {
                            ui.add_space(extra);
                        }
                        let footer = ui.vertical(|ui| {
                            ui.set_width(right_inner_w);
                            ui.set_max_width(right_inner_w);
                            footer_fn(ui);
                        });
                        right_natural = body_h + footer.response.rect.height();
                    } else {
                        right_natural = body_h;
                        let extra = (cache.target_h - right_natural).max(0.0);
                        if extra > 0.5 {
                            ui.add_space(extra);
                        }
                    }
                });
            },
        );
    });

    let next_target = left_natural.max(right_natural).ceil();
    let changed = (next_target - cache.target_h).abs() > 0.5
        || (left_natural - cache.left_natural).abs() > 0.5
        || (right_natural - cache.right_natural).abs() > 0.5;
    if changed {
        ui.ctx().memory_mut(|mem| {
            mem.data.insert_temp(
                cache_id,
                PairHeightCache {
                    target_h: next_target,
                    left_natural,
                    right_natural,
                },
            );
        });
        ui.ctx().request_repaint();
    }
}

/// 统一居中模态弹窗容器（全站所有弹窗——设置、关于、二次确认——遵循同一视觉与动画标准）：
/// - 从主窗口正中央向外缓出缩放（0.88 -> 1.0）+ 淡入（160ms，遵循系统「减少动画」降级为 0ms）；
/// - 动画完成后零空闲重绘；
/// - 支持 Esc、点击遮罩与右上角「关闭」按钮关闭，内置纵向滚动确保最小窗口下内容完整可操作。
#[allow(clippy::too_many_arguments)]
pub fn centered_modal(
    ctx: &egui::Context,
    p: Palette,
    m: DensityMetrics,
    id_salt: &str,
    title: &str,
    opened_at: Option<std::time::Instant>,
    target_w: f32,
    target_h: f32,
    add_contents: impl FnOnce(&mut Ui),
) -> bool {
    let mut should_close = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    let duration_secs = if crate::system::prefers_reduced_motion() {
        0.0_f32
    } else {
        0.16_f32
    };
    let raw_t = if duration_secs <= 0.0 {
        1.0
    } else {
        opened_at
            .map(|t0| (t0.elapsed().as_secs_f32() / duration_secs).clamp(0.0, 1.0))
            .unwrap_or(1.0)
    };
    if raw_t < 1.0 {
        ctx.request_repaint();
    }
    // 三次缓出曲线（ease-out cubic）：从中央平滑弹出，无弹跳过冲
    let ease = 1.0 - (1.0 - raw_t).powi(3);
    let scale = egui::lerp(0.88..=1.0, ease);

    let screen = ctx.viewport_rect();
    let scrim_alpha = (115.0 * ease).round() as u8;
    egui::Area::new(egui::Id::new(("modal_scrim", id_salt)))
        .fixed_pos(screen.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (_, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(scrim_alpha));
            if resp.clicked() {
                should_close = true;
            }
        });

    let clamped_w = target_w.min((screen.width() - 48.0).max(320.0));
    let clamped_h = target_h.min((screen.height() - 48.0).max(180.0));
    let cur_w = (clamped_w * scale).round();
    let cur_h = (clamped_h * scale).round();
    let modal_rect = egui::Rect::from_center_size(screen.center(), Vec2::new(cur_w, cur_h));

    egui::Area::new(egui::Id::new(("modal_dialog", id_salt)))
        .fixed_pos(modal_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.multiply_opacity(ease);
            egui::Frame::new()
                .fill(p.bg_surface)
                .stroke(Stroke::new(1.0, p.border_control))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    let inner_w = (cur_w - 40.0).max(260.0);
                    let inner_h = (cur_h - 40.0).max(120.0);
                    ui.set_min_size(Vec2::new(inner_w, inner_h));
                    ui.set_max_size(Vec2::new(inner_w, inner_h));
                    ui.vertical(|ui| {
                        ui.set_width(inner_w);
                        ui.horizontal(|ui| {
                            ui.set_width(inner_w);
                            ui.label(
                                egui::RichText::new(title)
                                    .size(m.h2_size)
                                    .color(p.text_primary),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let close_resp = styled_button(
                                        ui,
                                        p,
                                        m,
                                        "关闭",
                                        ButtonKind::Ghost,
                                        true,
                                        false,
                                    );
                                    close_resp.widget_info(|| {
                                        WidgetInfo::labeled(WidgetType::Button, true, "关闭弹窗")
                                    });
                                    if close_resp.clicked() {
                                        should_close = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(10.0);
                        let (sep, _) = ui.allocate_exact_size(
                            Vec2::new(inner_w, 1.0),
                            Sense::hover(),
                        );
                        ui.painter().line_segment(
                            [sep.left_center(), sep.right_center()],
                            Stroke::new(1.0, p.border_subtle),
                        );
                        ui.add_space(12.0);

                        egui::ScrollArea::vertical()
                            .id_salt(("modal_scroll", id_salt))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                add_contents(ui);
                            });
                    });
                });
        });

    should_close
}
