mod compiler;
mod course;
mod store;

use egui::{Color32, RichText};
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use store::{Progress, Store};

const BLUE: Color32 = Color32::from_rgb(93, 167, 255);
const MUTED: Color32 = Color32::from_rgb(166, 176, 193);
const GOOD: Color32 = Color32::from_rgb(116, 212, 157);
const WARN: Color32 = Color32::from_rgb(244, 190, 96);

struct Completed {
    id: String,
    source: String,
    result: Result<compiler::Report, String>,
}
struct App {
    lessons: Vec<course::Lesson>,
    progress: Progress,
    store: Option<Store>,
    selected: usize,
    query: String,
    status: Option<(String, bool)>,
    storage_error: Option<String>,
    pending: Option<mpsc::Receiver<Completed>>,
    dirty: Option<Instant>,
    hint: bool,
    solution: bool,
    reset: bool,
    curriculum: bool,
    tab: usize,
    screenshot_requested: bool,
    started: Instant,
}
impl App {
    fn new(ctx: &egui::Context, lessons: Vec<course::Lesson>) -> Self {
        style(ctx);
        let opened = store::home().and_then(Store::open);
        let (store, progress, storage_error) = match opened {
            Ok((s, p)) => (Some(s), p, None),
            Err(e) => (None, Progress::default(), Some(e)),
        };
        let selected = lessons
            .iter()
            .position(|l| l.id == progress.selected)
            .unwrap_or(0);
        Self {
            lessons,
            progress,
            store,
            selected,
            storage_error,
            query: String::new(),
            status: None,
            pending: None,
            dirty: None,
            hint: false,
            solution: false,
            reset: false,
            curriculum: true,
            tab: 0,
            screenshot_requested: false,
            started: Instant::now(),
        }
    }
    fn save(&mut self) {
        if let Some(store) = &self.store {
            match store.save(&self.progress) {
                Ok(()) => {
                    self.dirty = None;
                    self.storage_error = None;
                }
                Err(e) => {
                    self.storage_error = Some(format!(
                        "Draft could not be saved: {e}. Keep this window open and retry."
                    ));
                    self.dirty = None;
                }
            }
        }
    }
    fn select(&mut self, index: usize) {
        self.selected = index;
        self.progress.selected = self.lessons[index].id.clone();
        self.hint = false;
        self.solution = false;
        self.status = None;
        self.dirty = Some(Instant::now());
    }
    fn check(&mut self, ctx: &egui::Context) {
        if self.pending.is_some() {
            return;
        }
        self.save();
        let lesson = self.lessons[self.selected].clone();
        let source = self.progress.draft(&lesson).source.clone();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.status = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = compiler::check(&lesson, &source);
            let _ = tx.send(Completed {
                id: lesson.id,
                source,
                result,
            });
            ctx.request_repaint();
        });
    }
    fn receive(&mut self) {
        let result = self.pending.as_ref().map(|r| r.try_recv());
        match result {
            Some(Ok(done)) => {
                self.pending = None;
                match done.result {
                    Ok(report) => {
                        if let Some(draft) = self.progress.drafts.get_mut(&done.id)
                            && draft.source == done.source
                        {
                            if report.passed {
                                draft.passed_source = Some(done.source);
                            } else {
                                draft.passed_source = None;
                            }
                            if self.lessons[self.selected].id == done.id {
                                self.status = Some((report.text, report.passed));
                            }
                            self.dirty = Some(Instant::now());
                        } else if self.lessons[self.selected].id == done.id {
                            self.status = Some((
                                "Draft changed while checking. Check the current draft again."
                                    .into(),
                                false,
                            ));
                        }
                    }
                    Err(e) => {
                        if self.lessons[self.selected].id == done.id {
                            self.status = Some((e, false));
                        }
                    }
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.pending = None;
                self.status = Some((
                    "Compiler job ended unexpectedly. Retry Check.".into(),
                    false,
                ));
            }
            _ => {}
        }
    }
    fn theory(&mut self, ui: &mut egui::Ui, lesson: &course::Lesson) {
        ui.label(RichText::new("UNDERSTAND").color(BLUE).size(11.0));
        ui.add_space(8.0);
        for paragraph in lesson.concept.split("\n\n") {
            ui.label(paragraph);
            ui.add_space(12.0);
        }
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("YOUR LAB").color(BLUE).size(11.0));
        ui.add_space(8.0);
        ui.label(&lesson.task);
        ui.add_space(16.0);
        if ui
            .button(if self.hint {
                "Hide hint"
            } else {
                "Need a hint?"
            })
            .clicked()
        {
            self.hint = !self.hint;
        }
        if self.hint {
            ui.add_space(8.0);
            ui.label(RichText::new(&lesson.hint).color(WARN));
        }
        ui.add_space(12.0);
        if ui.button("View worked solution").clicked() {
            self.solution = true;
            self.progress.draft(lesson).solution_seen = true;
            self.dirty = Some(Instant::now());
        }
        ui.add_space(14.0);
        ui.small("Every lab checks type contracts. Selected function labs also run sample inputs in an isolated JavaScript context.");
        ui.hyperlink_to(
            "Official TypeScript Handbook",
            "https://www.typescriptlang.org/docs/handbook/intro.html",
        );
    }
    fn lab(&mut self, ui: &mut egui::Ui, lesson: &course::Lesson) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("answer.ts").monospace().color(BLUE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("strict · ES2022");
            });
        });
        let draft = self.progress.draft(lesson);
        let editor = egui::TextEdit::multiline(&mut draft.source)
            .font(egui::TextStyle::Monospace)
            .code_editor()
            .desired_width(f32::INFINITY)
            .desired_rows(16)
            .lock_focus(true)
            .id(egui::Id::new(("editor", &lesson.id)));
        let changed = ui
            .add_sized([ui.available_width(), 280.0], editor)
            .changed();
        if changed {
            self.dirty = Some(Instant::now());
            self.status = None;
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let check = ui.add_enabled(
                self.pending.is_none(),
                egui::Button::new(RichText::new("Check lab").color(Color32::BLACK)).fill(BLUE),
            );
            if check.clicked() {
                self.check(ui.ctx());
            }
            if self.pending.is_some() {
                ui.spinner();
                ui.label("Compiling your draft…");
            }
            if ui.button("Reset draft…").clicked() {
                self.reset = true;
            }
        });
        ui.add_space(12.0);
        let passed = self.progress.draft(lesson).passed();
        if let Some((text, ok)) = &self.status {
            egui::Frame::new()
                .fill(Color32::from_rgb(24, 29, 39))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(if *ok { "PASSED" } else { "NEEDS ATTENTION" })
                            .color(if *ok { GOOD } else { WARN }),
                    );
                    ui.add_space(6.0);
                    ui.add(egui::Label::new(RichText::new(text).monospace().size(12.0)).wrap());
                });
        } else {
            ui.label(
                RichText::new(if passed {
                    "Passed · this exact draft was checked"
                } else {
                    "Not checked · edit the starter, then Check lab"
                })
                .color(if passed { GOOD } else { MUTED }),
            );
        }
        if passed && self.selected + 1 < self.lessons.len() {
            ui.add_space(12.0);
            if ui.button("Continue to next lesson").clicked() {
                self.select(self.selected + 1);
            }
        }
        if passed && self.progress.complete() == self.lessons.len() {
            ui.add_space(8.0);
            ui.label(RichText::new("All labs passed. Revisit the projects, explain your design choices, and keep practicing.").color(GOOD));
        }
    }
    fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.receive();
        if self
            .dirty
            .is_some_and(|t| t.elapsed() > Duration::from_millis(700))
        {
            self.save();
        }
        if self.dirty.is_some() || self.pending.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)) {
            self.check(&ctx);
        }
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Lessons").clicked() {
                    self.curriculum = !self.curriculum;
                }
                ui.label(RichText::new("Type").size(23.0).strong());
                ui.label(RichText::new("Dojo").size(23.0).strong().color(BLUE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} / {} labs passed",
                            self.progress.complete(),
                            self.lessons.len()
                        ))
                        .color(MUTED),
                    );
                    ui.small("0.1 · Foundations to advanced");
                });
            });
            ui.add_space(8.0);
        });
        if let Some(error) = self.storage_error.clone() {
            egui::Panel::top("storage-error").show(ui, |ui| {
                ui.colored_label(WARN, error);
                if self.store.is_some() && ui.button("Retry save").clicked() {
                    self.save();
                }
            });
        }
        egui::Panel::bottom("footer").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.small("KEIKO FAMILY  /  Practice, understand, repeat.");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(if self.store.is_none() {
                        "Practice mode · saving unavailable"
                    } else if self.storage_error.is_some() {
                        "Save failed"
                    } else if self.dirty.is_some() {
                        "Saving…"
                    } else {
                        "Saved on this Mac"
                    });
                    ui.small("Cmd+Enter checks");
                });
            });
        });
        if self.curriculum {
            egui::Panel::left("curriculum")
                .default_size(248.0)
                .size_range(210.0..=320.0)
                .show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("THE LEARNING PATH").color(MUTED).size(11.0));
                    ui.add_space(8.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Find a lesson…")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(8.0);
                    let mut choose = None;
                    let mut section = String::new();
                    let query = self.query.to_lowercase();
                    egui::ScrollArea::vertical()
                        .id_salt("lessons")
                        .show(ui, |ui| {
                            for (i, lesson) in self.lessons.iter().enumerate() {
                                if !query.is_empty()
                                    && !format!("{} {}", lesson.title, lesson.section)
                                        .to_lowercase()
                                        .contains(&query)
                                {
                                    continue;
                                }
                                if section != lesson.section {
                                    section = lesson.section.clone();
                                    ui.add_space(12.0);
                                    ui.label(RichText::new(&section).color(BLUE).size(12.0));
                                    ui.add_space(4.0);
                                }
                                let passed = self
                                    .progress
                                    .drafts
                                    .get(&lesson.id)
                                    .is_some_and(|d| d.passed());
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(format!(
                                                "{:02}  {}{}",
                                                i + 1,
                                                lesson.title,
                                                if passed { "  [pass]" } else { "" }
                                            ))
                                            .size(13.0),
                                        )
                                        .selected(self.selected == i)
                                        .wrap(),
                                    )
                                    .clicked()
                                {
                                    choose = Some(i);
                                }
                            }
                            if section.is_empty() {
                                ui.label("No lessons match. Clear the search to see the path.");
                            }
                        });
                    if let Some(index) = choose {
                        self.select(index);
                    }
                });
        }
        egui::CentralPanel::default().show(ui, |ui| {
            let lesson = self.lessons[self.selected].clone();
            ui.add_space(12.0);
            ui.label(RichText::new(&lesson.section).color(BLUE).size(12.0));
            ui.heading(&lesson.title);
            ui.add_space(8.0);
            let width = ui.available_width();
            if width > 850.0 {
                ui.columns(2, |columns| {
                    egui::ScrollArea::vertical()
                        .id_salt("theory")
                        .show(&mut columns[0], |ui| self.theory(ui, &lesson));
                    egui::ScrollArea::vertical()
                        .id_salt("lab")
                        .show(&mut columns[1], |ui| self.lab(ui, &lesson));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.tab, 0, "Learn");
                    ui.selectable_value(&mut self.tab, 1, "Practice");
                });
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .id_salt("compact")
                    .show(ui, |ui| {
                        if self.tab == 0 {
                            self.theory(ui, &lesson);
                        } else {
                            self.lab(ui, &lesson);
                        }
                    });
            }
        });
        if self.solution {
            let lesson = self.lessons[self.selected].clone();
            egui::Window::new("Worked solution").open(&mut self.solution).default_width(600.0).show(&ctx,|ui|{ui.label("Read, close, then write your own answer. Viewing this does not pass the lab.");ui.add_space(10.0);egui::ScrollArea::both().show(ui,|ui|{ui.monospace(&lesson.solution);});});
        }
        if self.reset {
            egui::Window::new("Reset this draft?").collapsible(false).resizable(false).show(&ctx,|ui|{ui.label("Your current answer will be replaced by the starter. Other lessons stay saved.");ui.horizontal(|ui|{if ui.button("Keep my answer").clicked(){self.reset=false;}
if ui.button("Reset this lesson").clicked(){let l=&self.lessons[self.selected];let d=self.progress.draft(l);d.source=l.starter.clone();d.passed_source=None;self.status=None;self.dirty=Some(Instant::now());self.reset=false;}});});
        }
        self.screenshot(&ctx);
    }
    fn screenshot(&mut self, ctx: &egui::Context) {
        let Ok(path) = std::env::var("TYPEDOJO_SCREENSHOT") else {
            return;
        };
        let img = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(img) = img {
            let bytes: Vec<u8> = img.pixels.iter().flat_map(|p| p.to_array()).collect();
            if let Err(e) = image::save_buffer(
                &path,
                &bytes,
                img.size[0] as u32,
                img.size[1] as u32,
                image::ColorType::Rgba8,
            ) {
                eprintln!("Screenshot: {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if !self.screenshot_requested && self.started.elapsed() > Duration::from_millis(700)
        {
            self.screenshot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.dirty.is_some() {
            self.save();
        }
    }
}
fn style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for (name, bytes, family) in [
        (
            "inter",
            &include_bytes!("../assets/fonts/Inter-Regular.ttf")[..],
            egui::FontFamily::Proportional,
        ),
        (
            "mono",
            &include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")[..],
            egui::FontFamily::Monospace,
        ),
    ] {
        fonts
            .font_data
            .insert(name.into(), Arc::new(egui::FontData::from_static(bytes)));
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.into());
    }
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(16, 20, 28);
    visuals.override_text_color = Some(Color32::from_rgb(230, 235, 244));
    visuals.selection.bg_fill = Color32::from_rgb(35, 71, 112);
    visuals.hyperlink_color = BLUE;
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        s.spacing.button_padding = egui::vec2(10.0, 7.0);
        s.text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        s.text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));
    });
}
fn main() -> eframe::Result {
    let lessons = match course::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        }
    };
    if std::env::args().any(|a| a == "--verify-content") {
        if let Err(e) = compiler::verify(&lessons) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let size = std::env::var("TYPEDOJO_WINDOW")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        })
        .unwrap_or([1440.0, 900.0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("TypeDojo")
            .with_app_id("dev.bujosa.typedojo")
            .with_inner_size(size)
            .with_min_inner_size([760.0, 560.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "TypeDojo",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(&cc.egui_ctx, lessons)))),
    )
}
