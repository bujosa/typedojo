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
pub fn courtyard(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(include_bytes!("../assets/art/courtyard-banner-v1.png"))
        .ok()?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some(ctx.load_texture(
        "original-courtyard",
        egui::ColorImage::from_rgba_unmultiplied(size, &image),
        egui::TextureOptions::LINEAR,
    ))
}

pub fn blend_image_edge(painter: &egui::Painter, rect: egui::Rect) {
    let fade = egui::Rect::from_min_size(rect.min, egui::vec2(60.0, rect.height()));
    let mut mesh = egui::Mesh::default();
    for (pos, color) in [
        (fade.left_top(), PANEL),
        (fade.right_top(), Color32::TRANSPARENT),
        (fade.left_bottom(), PANEL),
        (fade.right_bottom(), Color32::TRANSPARENT),
    ] {
        mesh.colored_vertex(pos, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    painter.add(egui::Shape::mesh(mesh));
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

/// A compact editorial masthead that protects the working area on small screens.
pub fn masthead(
    ui: &mut egui::Ui,
    art: Option<&egui::TextureHandle>,
    section: &str,
    title: &str,
    index: usize,
    compact: bool,
    practice: bool,
) {
    let width = ui.available_width();
    let short = compact && practice;
    let height = if short { 82.0 } else { 108.0 };
    let (banner, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(banner, 10.0, PANEL);
    if let Some(art) = art {
        let natural_width = height * art.size_vec2().x / art.size_vec2().y;
        let art_width = (width * 0.40).min(390.0).min(natural_width);
        let rect = egui::Rect::from_min_size(
            egui::pos2(banner.right() - art_width, banner.top()),
            egui::vec2(art_width, height),
        );
        let uv = egui::Rect::from_min_max(
            egui::pos2((1.0 - art_width / natural_width).max(0.0), 0.0),
            egui::pos2(1.0, 1.0),
        );
        ui.painter().add(
            egui::epaint::RectShape::filled(
                rect,
                egui::CornerRadius {
                    nw: 0,
                    ne: 10,
                    sw: 0,
                    se: 10,
                },
                Color32::WHITE,
            )
            .with_texture(art.id(), uv),
        );
        blend_image_edge(ui.painter(), rect);
    }
    let text_rect = egui::Rect::from_min_size(
        banner.min + egui::vec2(20.0, 12.0),
        egui::vec2(width * 0.60 - 26.0, height - 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(text_rect), |ui| {
        eyebrow(ui, &section.to_uppercase(), ACCENT);
        ui.add_space(5.0);
        ui.label(RichText::new(title).font(display(if compact { 20.0 } else { 26.0 })));
        if !short {
            ui.add_space(5.0);
            ui.label(
                RichText::new(format!("LAB {:02}  /  Learn by building", index + 1))
                    .monospace()
                    .size(11.5)
                    .color(MUTED),
            );
        }
    });
}
