//! Searchable, keyboard-accessible commands for the native workbench.
use crate::design;
use egui::{RichText, Stroke};
#[derive(Clone, Copy)]
pub enum Action {
    Run,
    Format,
    Solution,
    Reset,
    Debug,
    Continue,
    Stop,
    Over,
    Into,
    Out,
    Check,
    Task,
    Answer,
    Playground,
    Find,
    Sidebar,
    Console,
    Problems,
    Larger,
    Smaller,
}
pub struct Command {
    pub action: Action,
    pub name: &'static str,
    pub shortcut: &'static str,
    pub enabled: bool,
}
#[derive(Default)]
pub struct Palette {
    open: bool,
    focus: bool,
    query: String,
    selected: usize,
}
impl Palette {
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn open(&mut self) {
        self.open = true;
        self.focus = true;
        self.query.clear();
        self.selected = 0;
    }
    pub fn show(&mut self, ctx: &egui::Context, commands: &[Command]) -> Option<Action> {
        if !self.open {
            return None;
        }
        let mut chosen = None;
        let response = egui::Modal::new(egui::Id::new("command-palette"))
            .frame(
                egui::Frame::new()
                    .fill(design::PANEL)
                    .stroke(Stroke::new(1.0, design::BORDER))
                    .corner_radius(12.0)
                    .inner_margin(18.0),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 80.0).clamp(280.0, 560.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Command palette").font(design::display(18.0)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Esc to close")
                                .size(11.0)
                                .color(design::MUTED),
                        );
                    });
                });
                ui.add_space(12.0);
                let field = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .id_salt("command-query")
                        .return_key(None)
                        .hint_text("Type a command…")
                        .desired_width(f32::INFINITY),
                );
                if self.focus {
                    field.request_focus();
                    self.focus = false;
                }
                if field.changed() {
                    self.selected = 0;
                }
                let query = self.query.trim().to_lowercase();
                let matches: Vec<_> = commands
                    .iter()
                    .filter(|command| {
                        query
                            .split_whitespace()
                            .all(|word| command.name.to_lowercase().contains(word))
                    })
                    .collect();
                self.selected = self.selected.min(matches.len().saturating_sub(1));
                let down =
                    ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
                let up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                if !matches.is_empty() {
                    if down {
                        self.selected = (self.selected + 1) % matches.len();
                    }
                    if up {
                        self.selected = if self.selected == 0 {
                            matches.len() - 1
                        } else {
                            self.selected - 1
                        };
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
                        && matches[self.selected].enabled
                    {
                        chosen = Some(matches[self.selected].action);
                    }
                }
                ui.add_space(10.0);
                egui::ScrollArea::vertical()
                    .id_salt("command-list")
                    .max_height((ctx.content_rect().height() - 270.0).clamp(150.0, 340.0))
                    .show(ui, |ui| {
                        if matches.is_empty() {
                            ui.add_space(14.0);
                            ui.label("No matching commands");
                            ui.label(
                                RichText::new("Try run, find, console or file.")
                                    .size(12.0)
                                    .color(design::MUTED),
                            );
                            ui.add_space(14.0);
                        }
                        for (index, command) in matches.iter().enumerate() {
                            let response = ui.add_enabled(
                                command.enabled,
                                egui::Button::new(RichText::new(command.name).size(13.0))
                                    .shortcut_text(command.shortcut)
                                    .fill(if self.selected == index {
                                        egui::Color32::from_rgb(30, 54, 80)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    })
                                    .stroke(Stroke::NONE)
                                    .corner_radius(4.0)
                                    .min_size(egui::vec2(ui.available_width(), 34.0)),
                            );
                            if response.clicked() {
                                chosen = Some(command.action);
                            }
                            if (up || down) && index == self.selected {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });
                ui.add_space(10.0);
                ui.separator();
                ui.label(
                    RichText::new("↑ ↓ navigate     Enter select     Cmd+Shift+P commands")
                        .size(11.0)
                        .color(design::MUTED),
                );
            });
        if chosen.is_some() || response.should_close() {
            self.open = false;
        }
        chosen
    }
}
