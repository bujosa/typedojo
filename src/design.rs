//! TypeDojo's original visual language: indigo ink, warm paper, and structural brackets.
use egui::{Color32, FontId, RichText, Stroke, Vec2};
pub const BG: Color32 = Color32::from_rgb(13, 21, 31);
pub const PANEL: Color32 = Color32::from_rgb(18, 29, 42);
pub const BORDER: Color32 = Color32::from_rgb(42, 58, 76);
pub const PAPER: Color32 = Color32::from_rgb(232, 226, 212);
pub const INK: Color32 = Color32::from_rgb(36, 45, 60);
pub const BLUE_INK: Color32 = Color32::from_rgb(30, 79, 145);
pub const ACCENT: Color32 = Color32::from_rgb(135, 186, 247);
pub const MUTED: Color32 = Color32::from_rgb(163, 180, 199);
pub const EDITOR: Color32 = Color32::from_rgb(12, 19, 29);
pub const SELECTED: Color32 = Color32::from_rgb(30, 54, 80);
pub const TEXT: Color32 = Color32::from_rgb(233, 237, 241);

pub fn display(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("display".into()))
}
pub fn eyebrow(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.label(RichText::new(text).monospace().size(11.5).color(color));
}
pub fn seal(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 8.0, BLUE_INK);
    let stroke = Stroke::new((size / 17.0).max(1.5), PAPER);
    let x = rect.left();
    let y = rect.top();
    for points in [
        vec![[0.36, 0.28], [0.25, 0.28], [0.25, 0.72], [0.36, 0.72]],
        vec![[0.64, 0.28], [0.75, 0.28], [0.75, 0.72], [0.64, 0.72]],
    ] {
        p.add(egui::Shape::line(
            points
                .iter()
                .map(|a| egui::pos2(x + size * a[0], y + size * a[1]))
                .collect(),
            stroke,
        ));
    }
    p.line_segment(
        [
            egui::pos2(x + size * 0.47, y + size * 0.66),
            egui::pos2(x + size * 0.56, y + size * 0.34),
        ],
        stroke,
    );
}
/// Six original chapter crests drawn on the same grid as the bracket seal.
pub fn chapter(ui: &mut egui::Ui, index: u8) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    let stroke = Stroke::new(1.4, ACCENT);
    match index {
        b'1' => {
            p.circle_stroke(c, 6.5, stroke);
            p.circle_filled(c, 2.0, ACCENT);
        }
        b'2' => {
            p.line_segment(
                [c + egui::vec2(-3.0, -6.0), c + egui::vec2(-7.0, -6.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-7.0, -6.0), c + egui::vec2(-7.0, 6.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-7.0, 6.0), c + egui::vec2(-3.0, 6.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(3.0, -6.0), c + egui::vec2(7.0, -6.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(7.0, -6.0), c + egui::vec2(7.0, 6.0)],
                stroke,
            );
            p.line_segment([c + egui::vec2(7.0, 6.0), c + egui::vec2(3.0, 6.0)], stroke);
        }
        b'3' => {
            for offset in [-3.0, 3.0] {
                p.rect_stroke(
                    egui::Rect::from_center_size(
                        c + egui::vec2(offset, offset),
                        egui::vec2(9.0, 9.0),
                    ),
                    1.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        b'4' => {
            for dy in [-5.0, 5.0] {
                p.line_segment([c + egui::vec2(-6.0, 0.0), c + egui::vec2(5.0, dy)], stroke);
                p.circle_filled(c + egui::vec2(6.0, dy), 1.5, ACCENT);
            }
        }
        b'5' => {
            p.circle_stroke(c + egui::vec2(-3.0, 0.0), 5.0, stroke);
            p.circle_stroke(c + egui::vec2(3.0, 0.0), 5.0, stroke);
        }
        _ => {
            p.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(14.0, 14.0)),
                1.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            p.line_segment(
                [c + egui::vec2(-4.0, 1.0), c + egui::vec2(-1.0, 4.0)],
                stroke,
            );
            p.line_segment(
                [c + egui::vec2(-1.0, 4.0), c + egui::vec2(5.0, -4.0)],
                stroke,
            );
        }
    }
}

/// Lesson-specific artwork stays separate from text and remains visible on compact windows.
pub fn masthead(
    ui: &mut egui::Ui,
    section: &str,
    title: &str,
    index: usize,
    art: Option<&egui::TextureHandle>,
    caption: &str,
    failed: bool,
) {
    let width = ui.available_width();
    let compact = width < 900.0;
    let height = if compact { 152.0 } else { 168.0 };
    let art_width = (width * 0.35).clamp(220.0, 390.0);
    let (banner, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(banner, 8.0, PANEL);
    let mut bounds = egui::Rect::from_min_max(
        egui::pos2(banner.right() - art_width, banner.top()),
        banner.right_bottom(),
    )
    .shrink(8.0);
    bounds.max.y -= 19.0;
    if let Some(art) = art {
        let size = art.size_vec2();
        let scale = (bounds.width() / size.x).min(bounds.height() / size.y);
        let rect = egui::Rect::from_center_size(bounds.center(), size * scale);
        ui.painter().image(
            art.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        let response = ui.interact(
            rect,
            ui.id().with(("lesson-art", index)),
            egui::Sense::hover(),
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, caption));
        let caption_rect = egui::Rect::from_min_max(
            egui::pos2(bounds.left(), bounds.bottom() + 4.0),
            egui::pos2(bounds.right(), banner.bottom() - 3.0),
        );
        ui.painter().with_clip_rect(caption_rect).text(
            egui::pos2(bounds.center().x, bounds.bottom() + 5.0),
            egui::Align2::CENTER_TOP,
            caption,
            FontId::proportional(10.5),
            MUTED,
        );
    } else {
        ui.scope_builder(egui::UiBuilder::new().max_rect(bounds), |ui| {
            ui.centered_and_justified(|ui| {
                if failed {
                    ui.label(
                        RichText::new("Illustration unavailable")
                            .size(11.0)
                            .color(MUTED),
                    );
                } else {
                    ui.spinner();
                }
            });
        });
    }
    let text_rect = egui::Rect::from_min_size(
        banner.min + egui::vec2(20.0, 18.0),
        egui::vec2(width - art_width - 34.0, height - 30.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(text_rect), |ui| {
        eyebrow(ui, &section.to_uppercase(), ACCENT);
        ui.add_space(8.0);
        ui.add(
            egui::Label::new(RichText::new(title).font(display(if compact { 24.0 } else { 29.0 })))
                .wrap(),
        );
        ui.add_space(9.0);
        ui.label(
            RichText::new(format!("LESSON {:02}  /  Learn by building", index + 1))
                .monospace()
                .size(11.5)
                .color(MUTED),
        );
    });
    ui.advance_cursor_after_rect(banner);
}

/// A navigation tab with an explicit active underline; never a completion indicator.
pub fn tab(ui: &mut egui::Ui, label: impl Into<String>, selected: bool) -> egui::Response {
    let response = ui.add(
        egui::Button::new(RichText::new(label.into()).size(12.5).color(if selected {
            TEXT
        } else {
            MUTED
        }))
        .selected(selected)
        .fill(if selected {
            SELECTED
        } else {
            Color32::TRANSPARENT
        })
        .stroke(Stroke::NONE)
        .corner_radius(4.0)
        .min_size(egui::vec2(0.0, 32.0)),
    );
    if selected {
        ui.painter().line_segment(
            [
                response.rect.left_bottom() + egui::vec2(7.0, 0.0),
                response.rect.right_bottom() - egui::vec2(7.0, 0.0),
            ],
            Stroke::new(2.0, ACCENT),
        );
    }
    response
}
