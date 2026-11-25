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
        ui.available_width().max(min_w)
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

/// 左侧导航项（带左侧 3px 强调色指示条 + 精确垂直居中 + AccessKit 选中语义）
pub fn nav_item(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    label: &str,
    selected: bool,
) -> Response {
    let desired = Vec2::new(ui.available_width(), m.nav_item_height);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            format!("导航：{label}"),
        )
    });

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let bg = if selected {
            p.bg_selected
        } else if hovered {
            p.bg_hover
        } else {
            Color32::TRANSPARENT
        };
        let fg = if selected || hovered {
            p.text_primary
        } else {
            p.text_secondary
        };

        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), bg);

        if selected {
            let ind_h = 18.0;
            let ind_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + 4.0, rect.center().y - ind_h / 2.0),
                Vec2::new(3.0, ind_h),
            );
            painter.rect_filled(ind_rect, CornerRadius::same(2), p.accent_primary);
        }

        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(7),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        painter.text(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(m.body_size),
            fg,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// 分段选择控件（水平与视觉垂直双居中 + AccessKit 语义）
pub fn segment_bar<T: Copy + PartialEq>(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    current: &mut T,
    options: &[(T, &str)],
    group_prefix: &str,
) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(p.bg_control_alt)
        .stroke(Stroke::new(1.0, p.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::same(3))
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
                    let size = Vec2::new((galley.size().x + 24.0).max(52.0), m.control_height - 6.0);
                    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                    resp.widget_info(|| {
                        WidgetInfo::selected(
                            WidgetType::RadioButton,
                            true,
                            selected,
                            format!("{group_prefix}：{label}"),
                        )
                    });

                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        let (bg, stroke, fg) = if selected {
                            (
                                p.bg_surface,
                                Stroke::new(1.0, p.border_control),
                                p.text_primary,
                            )
                        } else if resp.hovered() {
                            (p.bg_hover, Stroke::NONE, p.text_primary)
                        } else {
                            (Color32::TRANSPARENT, Stroke::NONE, p.text_secondary)
                        };
                        painter.rect(rect, CornerRadius::same(4), bg, stroke, StrokeKind::Inside);
                        if resp.has_focus() {
                            painter.rect_stroke(
                                rect.expand(2.0),
                                CornerRadius::same(5),
                                Stroke::new(2.0, p.focus_ring),
                                StrokeKind::Outside,
                            );
                        }
                        painter.text(rect.center(), Align2::CENTER_CENTER, label, font_id, fg);
                    }

                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !selected {
                        *current = val;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// 现代拨动开关（带滑轨圆点 + 居中对齐中文标签 + AccessKit Checkbox 语义）
pub fn toggle_switch(
    ui: &mut Ui,
    p: Palette,
    m: DensityMetrics,
    checked: &mut bool,
    label: &str,
) -> Response {
    let font_id = FontId::proportional(m.body_size);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_id.clone(), p.text_primary);
    let track_w: f32 = 40.0;
    let track_h: f32 = 22.0;
    let gap = 10.0;
    let total_w = track_w + gap + galley.size().x + 4.0;
    let total_h = track_h.max(galley.size().y + 4.0);

    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click());
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }

    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, *checked, label));

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let track_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - track_h / 2.0),
            Vec2::new(track_w, track_h),
        );
        let (track_bg, track_border, thumb_fill) = if *checked {
            let bg = if response.hovered() {
                p.accent_hover
            } else {
                p.accent_primary
            };
            (bg, bg, p.text_on_accent)
        } else {
            let bg = if response.hovered() {
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

        let thumb_r = 7.0;
        let thumb_x = if *checked {
            track_rect.right() - 4.0 - thumb_r
        } else {
            track_rect.left() + 4.0 + thumb_r
        };
        painter.circle_filled(egui::pos2(thumb_x, track_rect.center().y), thumb_r, thumb_fill);

        if response.has_focus() {
            painter.rect_stroke(
                track_rect.expand(2.5),
                CornerRadius::same(13),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        painter.text(
            egui::pos2(track_rect.right() + gap, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font_id,
            p.text_primary,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// 语义状态徽章（视觉居中胶囊）
pub fn status_badge(ui: &mut Ui, p: Palette, m: DensityMetrics, text: &str, kind: StatusKind) {
    let (fg, bg, bd) = status_colors(p, kind);
    let font_id = FontId::proportional(m.caption_size);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_id.clone(), fg);
    let size = Vec2::new(galley.size().x + 18.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(4),
            bg,
            Stroke::new(1.0, bd),
            StrokeKind::Inside,
        );
        painter.text(rect.center(), Align2::CENTER_CENTER, text, font_id, fg);
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

/// 核查记录列表四列表头（编号 / 文件名称与位置 / 大小 / 状态）
pub fn workbench_task_header(ui: &mut Ui, p: Palette, m: DensityMetrics, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 28.0), Sense::hover());
    if ui.is_rect_visible(rect) {
        let cols = workbench_cols(rect);
        let painter = ui.painter();
        let font = FontId::proportional(m.caption_size);
        let cy = rect.center().y;
        painter.text(
            egui::pos2(cols.id_left, cy),
            Align2::LEFT_CENTER,
            "编号",
            font.clone(),
            p.text_muted,
        );
        painter.text(
            egui::pos2(cols.name_left, cy),
            Align2::LEFT_CENTER,
            "文件名称与位置",
            font.clone(),
            p.text_muted,
        );
        painter.text(
            egui::pos2(cols.size_right, cy),
            Align2::RIGHT_CENTER,
            "大小",
            font.clone(),
            p.text_muted,
        );
        painter.text(
            egui::pos2(cols.status_center_x, cy),
            Align2::CENTER_CENTER,
            "状态",
            font,
            p.text_muted,
        );
        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, p.border_subtle),
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
    let row_h = if m.control_height <= 30.0 { 48.0 } else { 58.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, row_h), Sense::click());

    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            true,
            selected,
            format!("{id} {file_name} {status}"),
        )
    });

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let bg = if selected {
            p.bg_selected
        } else if hovered {
            p.bg_hover
        } else {
            Color32::TRANSPARENT
        };
        let border = if selected {
            p.accent_primary
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
            painter.rect_filled(ind_rect, CornerRadius::same(2), p.accent_primary);
        }

        if response.has_focus() {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(7),
                Stroke::new(2.0, p.focus_ring),
                StrokeKind::Outside,
            );
        }

        let cols = workbench_cols(rect);
        let cy = rect.center().y;

        // 列 0：编号
        painter.text(
            egui::pos2(cols.id_left, cy),
            Align2::LEFT_CENTER,
            id,
            FontId::monospace(m.caption_size),
            p.text_secondary,
        );

        // 列 1：文件名 + 目录（严格裁剪与单行省略，绝不越界重叠）
        let name_galley = layout_ellipsized(
            painter,
            file_name,
            FontId::proportional(m.body_size),
            p.text_primary,
            cols.name_width,
        );
        let folder_galley = layout_ellipsized(
            painter,
            folder,
            FontId::proportional(m.caption_size),
            p.text_muted,
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
        name_painter.galley(egui::pos2(cols.name_left, top_y), name_galley.clone(), p.text_primary);
        name_painter.galley(
            egui::pos2(cols.name_left, top_y + name_galley.size().y + line_gap),
            folder_galley,
            p.text_muted,
        );

        // 列 2：大小（右对齐等宽）
        painter.text(
            egui::pos2(cols.size_right, cy),
            Align2::RIGHT_CENTER,
            size.trim(),
            FontId::monospace(m.caption_size),
            p.text_secondary,
        );

        // 列 3：状态胶囊徽章（几何居中）
        let (fg, badge_bg, badge_bd) = status_colors(p, kind);
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

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
