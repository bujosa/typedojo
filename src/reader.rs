//! Structured reading with definitions, an explained example and a low-stakes prediction.
use crate::{course::Lesson, design, editor, learning::Guide};
use egui::{RichText, Stroke};
#[derive(Default)]
pub struct Reader {
    example: editor::Editor,
    prediction: Option<usize>,
    copied: bool,
}
pub enum Action {
    Practice,
    Prerequisite(usize),
}
impl Reader {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        guide: &Guide,
        lesson: &Lesson,
        lessons: &[Lesson],
    ) -> Option<Action> {
        let mut action = None;
        egui::Frame::new()
            .fill(design::PANEL)
            .stroke(Stroke::new(1.0, design::BORDER))
            .corner_radius(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                design::eyebrow(ui, "WHAT YOU WILL LEARN", design::ACCENT);
                ui.label(
                    RichText::new(&guide.outcome)
                        .size(17.0)
                        .line_height(Some(24.0)),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(if guide.prerequisites.is_empty() {
                            "Starts from zero"
                        } else {
                            "Builds on:"
                        })
                        .size(11.5)
                        .color(design::MUTED),
                    );
                    for prerequisite in &guide.prerequisites {
                        if let Some((index, base)) = lessons
                            .iter()
                            .enumerate()
                            .find(|(_, l)| &l.id == prerequisite)
                            && ui.small_button(&base.title).clicked()
                        {
                            action = Some(Action::Prerequisite(index));
                        }
                    }
                });
            });
        ui.add_space(14.0);
        let wide = ui.available_width() > 900.0;
        let height = ui.available_height();
        if wide {
            let left = (ui.available_width() * 0.43).clamp(300.0, 500.0);
            let right = ui.available_width() - left - 20.0;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 20.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(left, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width(left);
                        egui::ScrollArea::vertical()
                            .id_salt("learning-definitions")
                            .max_height(height)
                            .show(ui, |ui| Self::definitions(ui, guide, lesson));
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(right, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width(right);
                        egui::ScrollArea::vertical()
                            .id_salt("learning-example")
                            .max_height(height)
                            .show(ui, |ui| {
                                if self.example(ui, guide) {
                                    action = Some(Action::Practice);
                                }
                            });
                    },
                );
            });
        } else {
            egui::ScrollArea::vertical()
                .id_salt("learning-compact")
                .show(ui, |ui| {
                    Self::definitions(ui, guide, lesson);
                    ui.add_space(20.0);
                    if self.example(ui, guide) {
                        action = Some(Action::Practice);
                    }
                });
        }
        action
    }
    fn definitions(ui: &mut egui::Ui, guide: &Guide, lesson: &Lesson) {
        design::eyebrow(ui, "01 / UNDERSTAND THE BUILDING BLOCKS", design::MUTED);
        ui.add_space(12.0);
        for term in &guide.vocabulary {
            ui.label(
                RichText::new(&term.term)
                    .size(14.5)
                    .strong()
                    .color(design::ACCENT),
            );
            ui.label(
                RichText::new(&term.meaning)
                    .size(14.0)
                    .line_height(Some(21.0)),
            );
            ui.add_space(14.0);
        }
        egui::CollapsingHeader::new("More detail and lab context").show(ui, |ui| {
            for paragraph in lesson.concept.split("\n\n") {
                ui.label(RichText::new(paragraph).size(13.0).line_height(Some(20.0)));
                ui.add_space(10.0);
            }
        });
        egui::CollapsingHeader::new("Transfer to the independent lab").show(ui, |ui| {
            for item in &guide.transfer {
                ui.label(RichText::new(format!("• {item}")).size(12.5));
            }
        });
        ui.add_space(12.0);
        ui.hyperlink_to(
            "TypeScript Handbook",
            "https://www.typescriptlang.org/docs/handbook/intro.html",
        );
    }
    fn example(&mut self, ui: &mut egui::Ui, guide: &Guide) -> bool {
        let mut practice = false;
        design::eyebrow(ui, "02 / FOLLOW A WORKED EXAMPLE", design::MUTED);
        ui.add_space(12.0);
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(12, 19, 29))
            .stroke(Stroke::new(1.0, design::BORDER))
            .corner_radius(8.0)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                egui::ScrollArea::horizontal()
                    .id_salt("example-code")
                    .show(ui, |ui| self.example.reference(ui, &guide.example));
            });
        if ui
            .small_button(if self.copied {
                "Copied"
            } else {
                "Copy example"
            })
            .clicked()
        {
            ui.ctx().copy_text(guide.example.clone());
            self.copied = true;
        }
        ui.add_space(12.0);
        for (index, step) in guide.explanation.iter().enumerate() {
            ui.label(
                RichText::new(format!("{}. {step}", index + 1))
                    .size(13.5)
                    .line_height(Some(21.0)),
            );
            ui.add_space(10.0);
        }
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);
        design::eyebrow(ui, "03 / CHECK YOUR UNDERSTANDING", design::MUTED);
        ui.add_space(8.0);
        ui.label(
            RichText::new(&guide.prediction.question)
                .size(14.0)
                .strong(),
        );
        for (index, choice) in guide.prediction.choices.iter().enumerate() {
            if ui
                .add(
                    egui::Button::new(RichText::new(choice).size(13.0))
                        .fill(if self.prediction == Some(index) {
                            egui::Color32::from_rgb(30, 54, 80)
                        } else {
                            design::PANEL
                        })
                        .min_size(egui::vec2(ui.available_width(), 30.0))
                        .wrap(),
                )
                .clicked()
            {
                self.prediction = Some(index);
            }
        }
        if let Some(index) = self.prediction {
            let correct = index == guide.prediction.correct;
            ui.add_space(8.0);
            ui.label(
                RichText::new(if correct {
                    "Correct reasoning"
                } else {
                    "Try the reasoning again"
                })
                .strong()
                .size(13.0)
                .color(if correct {
                    egui::Color32::from_rgb(116, 212, 157)
                } else {
                    egui::Color32::from_rgb(229, 190, 133)
                }),
            );
            ui.label(
                RichText::new(&guide.prediction.explanation)
                    .size(13.0)
                    .line_height(Some(20.0)),
            );
        }
        ui.add_space(18.0);
        if ui
            .add(
                egui::Button::new(RichText::new("Start guided practice").color(design::INK))
                    .fill(design::ACCENT),
            )
            .clicked()
        {
            practice = true;
        }
        ui.label(RichText::new("Try one smaller task, then transfer the idea to the independent lab. Reading and predictions do not mark a lab as passed.").size(11.5).color(design::MUTED));
        practice
    }
}
