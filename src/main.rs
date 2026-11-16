mod art;
mod commands;
mod compiler;
mod completion;
mod course;
mod design;
mod editor;
mod engine;
mod formatter;
mod ide;
mod learning;
mod reader;
mod store;
mod suggest;
mod tooling;

use egui::{Color32, RichText};
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use store::{Progress, Store};

const BLUE: Color32 = design::ACCENT;
const MUTED: Color32 = design::MUTED;
const GOOD: Color32 = Color32::from_rgb(116, 212, 157);
const WARN: Color32 = Color32::from_rgb(229, 190, 133);

struct Completed {
    id: String,
    source: String,
    result: Result<compiler::Report, String>,
    guided: bool,
}
struct FormatJob {
    id: String,
    file: String,
    source: String,
    points: std::collections::BTreeSet<usize>,
    result: Result<formatter::Formatted, String>,
}
struct App {
    illustrations: art::Illustrations,
    reader: reader::Reader,
    guides: Vec<learning::Guide>,
    workbench: ide::Workbench,
    reference_editor: editor::Editor,
    lessons: Vec<course::Lesson>,
    progress: Progress,
    store: Option<Store>,
    selected: usize,
    query: String,
    status: Option<(String, bool)>,
    storage_error: Option<String>,
    pending: Option<mpsc::Receiver<Completed>>,
    dirty: Option<Instant>,
    solution: bool,
    solution_copied: bool,
    reset: bool,
    curriculum: bool,
    open_sections: std::collections::HashSet<String>,
    focus: bool,
    path_before_focus: bool,
    stage: usize,
    format_pending: Option<mpsc::Receiver<FormatJob>>,
    format_notice: Option<(String, bool)>,
    screenshot_requested: bool,
    started: Instant,
}
impl App {
    fn new(
        ctx: &egui::Context,
        lessons: Vec<course::Lesson>,
        guides: Vec<learning::Guide>,
        initial_width: f32,
    ) -> Self {
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
        let open_sections = std::iter::once(lessons[selected].section.clone()).collect();
        Self {
            open_sections,
            focus: false,
            path_before_focus: true,
            illustrations: art::Illustrations::default(),
            reader: reader::Reader::default(),
            guides,
            workbench: ide::Workbench::default(),
            reference_editor: editor::Editor::default(),
            lessons,
            progress,
            store,
            selected,
            storage_error,
            query: String::new(),
            status: None,
            pending: None,
            dirty: None,
            solution: false,
            solution_copied: false,
            reset: false,
            curriculum: initial_width > 1000.0,
            stage: 0,
            format_pending: None,
            format_notice: None,
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
        self.workbench.select(&self.lessons[index].id);
        self.selected = index;
        self.stage = 0;
        self.reader = reader::Reader::default();
        self.format_notice = None;
        self.open_sections
            .insert(self.lessons[index].section.clone());
        self.progress.selected = self.lessons[index].id.clone();
        self.solution = false;
        self.status = None;
        self.dirty = Some(Instant::now());
    }
    fn active_lesson(&self) -> course::Lesson {
        if self.stage == 1 {
            self.guides[self.selected].lesson(&self.lessons[self.selected])
        } else {
            self.lessons[self.selected].clone()
        }
    }
    fn active_draft(&mut self, lesson: &course::Lesson) -> &mut store::Draft {
        if self.stage == 1 {
            self.progress
                .guided_draft(lesson, &self.guides[self.selected].practice.playground)
        } else {
            self.progress.draft(lesson)
        }
    }
    fn set_stage(&mut self, stage: usize) {
        self.workbench.stop();
        self.stage = stage.min(2);
        self.status = None;
        self.format_notice = None;
        self.workbench.select(&self.active_lesson().id);
    }
    fn check(&mut self, ctx: &egui::Context) {
        if self.pending.is_some() || self.format_pending.is_some() || self.stage == 0 {
            return;
        }
        let lesson = self.active_lesson();
        let source = self.active_draft(&lesson).source.clone();
        let guided = self.stage == 1;
        self.save();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.status = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = compiler::check(&lesson, &source).map(|mut report|{
                if guided && !report.passed {report.text=format!("Practice is not solved yet.\n\nNext step: {}\n\nCompiler details (contract.ts compares your exports with the task):\n{}",lesson.hint,report.text);}
                report
            });
            let _ = tx.send(Completed {
                id: lesson.id,
                source,
                result,
                guided,
            });
            ctx.request_repaint();
        });
    }
    fn receive(&mut self) {
        let result = self.pending.as_ref().map(|r| r.try_recv());
        match result {
            Some(Ok(done)) => {
                self.pending = None;
                let current = self.active_lesson().id == done.id;
                match done.result {
                    Ok(report) => {
                        let drafts = if done.guided {
                            &mut self.progress.guided
                        } else {
                            &mut self.progress.drafts
                        };
                        if let Some(draft) = drafts.get_mut(&done.id)
                            && draft.source == done.source
                        {
                            draft.passed_source = if report.passed {
                                Some(done.source)
                            } else {
                                None
                            };
                            if current {
                                self.status = Some((report.text, report.passed));
                            }
                            self.dirty = Some(Instant::now());
                        } else if current {
                            self.status = Some((
                                "Draft changed while checking. Check the current draft again."
                                    .into(),
                                false,
                            ));
                        }
                    }
                    Err(error) => {
                        if current {
                            self.status = Some((error, false));
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
    fn format(&mut self, ctx: &egui::Context, file: String) {
        if self.stage == 0 || self.format_pending.is_some() || self.pending.is_some() {
            return;
        }
        let lesson = self.active_lesson();
        let draft = self.active_draft(&lesson);
        let source = if file == "answer.ts" {
            draft.source.clone()
        } else if file == "playground.ts" {
            draft
                .playground
                .clone()
                .unwrap_or_else(|| engine::PLAYGROUND.into())
        } else {
            return;
        };
        let points = draft.breakpoints.get(&file).cloned().unwrap_or_default();
        let (tx, rx) = mpsc::channel();
        self.format_pending = Some(rx);
        self.format_notice = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = formatter::format(&source, &points);
            let _ = tx.send(FormatJob {
                id: lesson.id,
                file,
                source,
                points,
                result,
            });
            ctx.request_repaint();
        });
    }
    fn receive_format(&mut self, ctx: &egui::Context) {
        let result = self.format_pending.as_ref().map(|r| r.try_recv());
        match result {
            Some(Ok(done)) => {
                self.format_pending = None;
                if self.active_lesson().id != done.id
                    || self.stage == 0
                    || self.workbench.active_file() != done.file
                {
                    self.format_notice = Some((
                        "File or stage changed. Format the current file again.".into(),
                        false,
                    ));
                    return;
                }
                match done.result {
                    Ok(formatted) => {
                        let lesson = self.active_lesson();
                        let draft = self.active_draft(&lesson);
                        let source = if done.file == "answer.ts" {
                            &mut draft.source
                        } else {
                            draft
                                .playground
                                .get_or_insert_with(|| engine::PLAYGROUND.into())
                        };
                        if *source != done.source
                            || draft
                                .breakpoints
                                .get(&done.file)
                                .cloned()
                                .unwrap_or_default()
                                != done.points
                        {
                            self.format_notice = Some((
                                "Code or breakpoints changed while formatting. Format again."
                                    .into(),
                                false,
                            ));
                            return;
                        }
                        let changed = *source != formatted.source;
                        if changed {
                            editor::Editor::record_format(
                                ctx,
                                &format!("{}:{}", done.id, done.file),
                                source,
                                &formatted.source,
                            );
                        }
                        *source = formatted.source;
                        draft.breakpoints.insert(done.file, formatted.breakpoints);
                        if changed {
                            self.dirty = Some(Instant::now());
                            self.status = None;
                        }
                        self.format_notice = Some((
                            format!(
                                "Prettier {} · {}",
                                formatter::VERSION,
                                if changed {
                                    "Formatted. Check this revision again."
                                } else {
                                    "Already formatted."
                                }
                            ),
                            true,
                        ));
                    }
                    Err(error) => self.format_notice = Some((error, false)),
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.format_pending = None;
                self.format_notice = Some((
                    "Formatter stopped unexpectedly. Your code was preserved.".into(),
                    false,
                ));
            }
            _ => {}
        }
    }
    fn path(&mut self, ui: &mut egui::Ui, compact: bool) {
        if !compact {
            ui.add_space(12.0);
            design::eyebrow(ui, "LEARNING PATH", MUTED);
            ui.add_space(10.0);
        }
        ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text(format!("Search {} lessons", self.lessons.len()))
                .desired_width(f32::INFINITY),
        );
        ui.add_space(12.0);
        let query = self.query.trim().to_lowercase();
        let mut choose = None;
        let mut matches = 0;
        let mut sections = Vec::<String>::new();
        for lesson in &self.lessons {
            if sections.last() != Some(&lesson.section) {
                sections.push(lesson.section.clone());
            }
        }
        egui::ScrollArea::vertical()
            .id_salt("lessons")
            .show(ui, |ui| {
                for section in sections {
                    let indices: Vec<usize> = self
                        .lessons
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| l.section == section)
                        .map(|(i, _)| i)
                        .collect();
                    let visible: Vec<usize> = indices
                        .iter()
                        .copied()
                        .filter(|i| {
                            query.is_empty()
                                || format!("{} {}", self.lessons[*i].title, section)
                                    .to_lowercase()
                                    .contains(&query)
                        })
                        .collect();
                    if visible.is_empty() {
                        continue;
                    }
                    matches += visible.len();
                    let done = indices
                        .iter()
                        .filter(|i| {
                            self.progress
                                .drafts
                                .get(&self.lessons[**i].id)
                                .is_some_and(|d| d.passed())
                        })
                        .count();
                    let open = !query.is_empty() || self.open_sections.contains(&section);
                    ui.add_space(6.0);
                    let title = section
                        .split_once(" · ")
                        .map(|(_, name)| name)
                        .unwrap_or(&section);
                    ui.horizontal(|ui| {
                        design::chapter(ui, section.as_bytes().get(1).copied().unwrap_or(b'1'));
                        let button = ui.add(
                            egui::Button::new(
                                RichText::new(format!(
                                    "{} {}",
                                    if !query.is_empty() {
                                        ">"
                                    } else if open {
                                        "−"
                                    } else {
                                        "+"
                                    },
                                    title
                                ))
                                .size(12.0)
                                .color(if open {
                                    BLUE
                                } else {
                                    MUTED
                                }),
                            )
                            .frame(false)
                            .wrap(),
                        );
                        if button.clicked() {
                            if !query.is_empty() {
                                self.query.clear();
                                self.open_sections.insert(section.clone());
                            } else if self.open_sections.contains(&section) {
                                self.open_sections.remove(&section);
                            } else {
                                self.open_sections.insert(section.clone());
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{done}/{}", indices.len()))
                                    .monospace()
                                    .size(11.5)
                                    .color(MUTED),
                            );
                        });
                    });
                    ui.add(
                        egui::ProgressBar::new(done as f32 / indices.len() as f32)
                            .desired_width(ui.available_width())
                            .desired_height(2.0)
                            .fill(GOOD),
                    );
                    ui.add_space(6.0);
                    if open {
                        for i in visible {
                            let l = &self.lessons[i];
                            let passed =
                                self.progress.drafts.get(&l.id).is_some_and(|d| d.passed());
                            let selected = self.selected == i;
                            let response = ui.add(
                                egui::Button::new(
                                    RichText::new(format!(
                                        "{:02}  {}{}",
                                        i + 1,
                                        l.title,
                                        if passed { "  · Passed" } else { "" }
                                    ))
                                    .size(12.5),
                                )
                                .fill(if selected {
                                    Color32::from_rgb(30, 54, 80)
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(ui.available_width(), 34.0))
                                .wrap(),
                            );
                            if selected {
                                ui.painter().line_segment(
                                    [
                                        response.rect.left_top() + egui::vec2(1.0, 5.0),
                                        response.rect.left_bottom() + egui::vec2(1.0, -5.0),
                                    ],
                                    egui::Stroke::new(2.0, BLUE),
                                );
                            }
                            if response.clicked() {
                                choose = Some(i);
                            }
                        }
                    }
                    ui.add_space(8.0);
                }
                if matches == 0 {
                    ui.add_space(16.0);
                    ui.label("No lessons found");
                    ui.label(
                        RichText::new("Try a topic, such as generics or arrays.")
                            .size(12.0)
                            .color(MUTED),
                    );
                    if ui.button("Clear search").clicked() {
                        self.query.clear();
                    }
                }
            });
        if let Some(index) = choose {
            self.select(index);
            if compact {
                self.curriculum = false;
            }
        }
    }
    fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.receive();
        self.receive_format(&ctx);
        self.workbench.poll(&ctx);
        if self
            .dirty
            .is_some_and(|t| t.elapsed() > Duration::from_millis(700))
        {
            self.save();
        }
        if self.dirty.is_some() || self.pending.is_some() || self.format_pending.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if !self.solution
            && !self.reset
            && self.stage > 0
            && self.format_pending.is_none()
            && self.workbench.allows_check()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter))
        {
            self.check(&ctx);
        }
        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(design::BG)
                    .inner_margin(if self.focus {
                        egui::Margin::symmetric(12, 6)
                    } else {
                        egui::Margin::symmetric(20, 12)
                    }),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.focus {
                        ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
                    }
                    design::seal(ui, if self.focus { 24.0 } else { 34.0 });
                    ui.label(
                        RichText::new("TypeDojo").font(design::display(if self.focus {
                            17.0
                        } else {
                            23.0
                        })),
                    );
                    ui.add_space(14.0);
                    if ui
                        .button(if self.curriculum {
                            "Hide path"
                        } else {
                            "Learning path"
                        })
                        .clicked()
                    {
                        self.curriculum = !self.curriculum;
                    }
                    if ui
                        .button(if self.focus {
                            "Study view"
                        } else {
                            "Focus on code"
                        })
                        .clicked()
                    {
                        self.focus = !self.focus;
                        if self.focus {
                            if self.stage == 0 {
                                self.set_stage(2);
                            }
                            self.path_before_focus = self.curriculum;
                            self.curriculum = false;
                        } else {
                            self.curriculum = self.path_before_focus;
                            self.workbench.stop();
                        }
                    }
                    if self.focus && ui.available_width() > 620.0 {
                        ui.label(
                            RichText::new(&self.lessons[self.selected].title)
                                .size(12.0)
                                .color(MUTED),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let done = self.progress.complete();
                        ui.label(
                            RichText::new(format!("{done} / {} labs", self.lessons.len()))
                                .monospace()
                                .size(12.0)
                                .color(MUTED),
                        );
                        ui.add(
                            egui::ProgressBar::new(done as f32 / self.lessons.len() as f32)
                                .desired_width(80.0)
                                .desired_height(4.0)
                                .fill(GOOD),
                        );
                    });
                });
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
                ui.label(
                    RichText::new("KEIKO  /  TYPESCRIPT WORKSPACE")
                        .monospace()
                        .size(11.5)
                        .color(MUTED),
                );
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
        let compact = ui.available_width() < 1000.0;
        if self.curriculum && !compact && !self.focus {
            egui::Panel::left("curriculum")
                .default_size(256.0)
                .size_range(220.0..=320.0)
                .show(ui, |ui| self.path(ui, false));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            let base = self.lessons[self.selected].clone();
            let guide = self.guides[self.selected].clone();
            let guided_passed = self
                .progress
                .guided
                .get(&format!("guided-{}", base.id))
                .is_some_and(|draft| draft.passed());
            let lab_passed = self
                .progress
                .drafts
                .get(&base.id)
                .is_some_and(|draft| draft.passed());
            egui::Panel::top("learning-stages")
                .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(6, 8)))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (stage, label) in [
                            (0, "1  Learn".to_string()),
                            (
                                1,
                                format!(
                                    "2  Guided practice{}",
                                    if guided_passed { " · Passed" } else { "" }
                                ),
                            ),
                            (
                                2,
                                format!(
                                    "3  Independent lab{}",
                                    if lab_passed { " · Passed" } else { "" }
                                ),
                            ),
                        ] {
                            if design::tab(ui, label, self.stage == stage).clicked() {
                                self.set_stage(stage);
                            }
                        }
                        if self.stage == 1
                            && guided_passed
                            && ui.button("Continue to lab").clicked()
                        {
                            self.set_stage(2);
                        }
                        if self.stage == 2
                            && lab_passed
                            && self.selected + 1 < self.lessons.len()
                            && ui.button("Next lesson").clicked()
                        {
                            self.select(self.selected + 1);
                        }
                    });
                });
            if self.stage == 0 {
                let illustration = self.illustrations.get(&ctx, &base.id);
                design::masthead(
                    ui,
                    &base.section,
                    &base.title,
                    self.selected,
                    illustration.as_ref(),
                    art::caption(&base.id),
                    self.illustrations.failed(&base.id),
                );
                ui.add_space(12.0);
                if let Some(action) = self.reader.show(ui, &guide, &base, &self.lessons) {
                    match action {
                        reader::Action::Practice => self.set_stage(1),
                        reader::Action::Prerequisite(index) => self.select(index),
                    }
                }
            } else {
                let lesson = self.active_lesson();
                let guided = self.stage == 1;
                let draft = if guided {
                    self.progress
                        .guided_draft(&lesson, &guide.practice.playground)
                } else {
                    self.progress.draft(&lesson)
                };
                let initialized = draft.playground.is_none();
                if initialized {
                    draft.playground = Some(engine::PLAYGROUND.into());
                }
                let previous_file = self.workbench.active_file().to_owned();
                let response = self.workbench.show(
                    ui,
                    &lesson,
                    draft,
                    self.pending.is_some(),
                    self.status.as_ref().map(|(text, _)| text.as_str()),
                    ide::Context {
                        guided,
                        formatting: self.format_pending.is_some(),
                        format_notice: self
                            .format_notice
                            .as_ref()
                            .map(|(text, ok)| (text.as_str(), *ok)),
                    },
                );
                if self.workbench.active_file() != previous_file {
                    self.format_notice = None;
                }
                if response.changed || initialized {
                    self.dirty = Some(Instant::now());
                }
                if response.changed {
                    self.status = None;
                    self.format_notice = None;
                }
                if response.check {
                    self.check(&ctx);
                }
                if let Some(file) = response.format {
                    self.format(&ctx, file);
                }
                if response.solution {
                    self.solution = true;
                    self.solution_copied = false;
                    self.active_draft(&lesson).solution_seen = true;
                    self.dirty = Some(Instant::now());
                }
                if response.reset {
                    self.reset = true;
                }
            }
        });
        if self.curriculum && (compact || self.focus) {
            let mut open = true;
            let mut close_path = false;
            egui::Window::new("Learning path")
                .open(&mut open)
                .title_bar(false)
                .frame(
                    egui::Frame::new()
                        .fill(design::PANEL)
                        .stroke(egui::Stroke::new(1.0, design::BORDER))
                        .corner_radius(10.0)
                        .inner_margin(16.0),
                )
                .collapsible(false)
                .resizable(false)
                .fixed_pos(egui::pos2(16.0, 76.0))
                .default_width(300.0)
                .max_height((ctx.content_rect().height() - 110.0).max(200.0))
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Learning path").font(design::display(18.0)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_path = true;
                            }
                        });
                    });
                    ui.add_space(12.0);
                    self.path(ui, true);
                });
            if !open || close_path {
                self.curriculum = false;
            }
        }
        if self.solution {
            let lesson = self.active_lesson();
            let mut close = false;
            let response=egui::Modal::new(egui::Id::new("worked-solution"))
                .frame(egui::Frame::new().fill(design::PANEL).stroke(egui::Stroke::new(1.0,design::BORDER)).corner_radius(12.0).inner_margin(24.0))
                .show(&ctx,|ui|{
                    ui.set_width((ctx.content_rect().width()-80.0).clamp(280.0,680.0));
                    ui.horizontal(|ui|{ui.label(RichText::new("Worked solution").font(design::display(22.0)));ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("Close").clicked(){close=true;}});});
                    ui.add_space(6.0);ui.label(RichText::new(&lesson.title).size(13.0).color(MUTED));ui.add_space(14.0);
                    ui.label("Read the approach, then write your own answer. Viewing a solution does not pass the lab.");ui.add_space(18.0);
                    egui::Frame::new().fill(Color32::from_rgb(12,19,29)).corner_radius(7.0).inner_margin(16.0).show(ui,|ui|{
                        ui.set_min_width(ui.available_width());egui::ScrollArea::both().max_height((ctx.content_rect().height()*0.5).max(140.0)).show(ui,|ui|self.reference_editor.reference(ui,&lesson.solution));
                    });
                    ui.add_space(16.0);if ui.button(if self.solution_copied {"Copied"}else{"Copy code"}).clicked(){ui.ctx().copy_text(lesson.solution.clone());self.solution_copied=true;}
                });
            if close || response.should_close() {
                self.solution = false;
            }
        }
        if self.reset {
            let mut cancel = false;
            let mut reset = false;
            let response=egui::Modal::new(egui::Id::new("reset-answer"))
                .frame(egui::Frame::new().fill(design::PANEL).stroke(egui::Stroke::new(1.0,design::BORDER)).corner_radius(12.0).inner_margin(24.0))
                .show(&ctx,|ui|{
                    ui.set_width((ctx.content_rect().width()-80.0).clamp(280.0,420.0));
                    ui.label(RichText::new("Reset this answer?").font(design::display(22.0)));ui.add_space(12.0);
                    ui.label("Replace your current draft with the lesson starter. Other lessons keep their saved answers and progress.");ui.add_space(22.0);
                    ui.horizontal(|ui|{if ui.button("Keep editing").clicked(){cancel=true;}
if ui.add(egui::Button::new(RichText::new("Reset answer").color(design::TEXT)).fill(Color32::from_rgb(137,59,55))).clicked(){reset=true;}});
                });
            if reset {
                let lesson = self.active_lesson();
                let d = self.active_draft(&lesson);
                d.source = lesson.starter.clone();
                d.passed_source = None;
                self.status = None;
                self.dirty = Some(Instant::now());
            }
            if reset || cancel || response.should_close() {
                self.reset = false;
            }
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
    fonts.font_data.insert(
        "display".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/SpaceGrotesk-Bold.ttf"
        ))),
    );
    fonts.families.insert(
        egui::FontFamily::Name("display".into()),
        vec!["display".into(), "inter".into()],
    );
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = design::BG;
    visuals.override_text_color = Some(design::TEXT);
    visuals.selection.bg_fill = Color32::from_rgb(35, 71, 112);
    visuals.hyperlink_color = BLUE;
    visuals.extreme_bg_color = Color32::from_rgb(10, 17, 25);
    visuals.window_fill = design::PANEL;
    visuals.widgets.inactive.weak_bg_fill = design::PANEL;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, design::BORDER);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(36, 54, 74);
    for w in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        w.corner_radius = egui::CornerRadius::same(6);
    }
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|s| {
        s.spacing.scroll.fade.strength = 0.0;
        s.spacing.item_spacing = egui::vec2(12.0, 8.0);
        s.spacing.button_padding = egui::vec2(10.0, 7.0);
        s.text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        s.text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));
    });
}
fn main() -> eframe::Result {
    if let Some(arg) = std::env::args().skip(1).find(|arg| {
        arg.starts_with("--verify-")
            && ![
                "--verify-art",
                "--verify-content",
                "--verify-learning",
                "--verify-completion",
            ]
            .contains(&arg.as_str())
    }) {
        eprintln!("Unknown validation option: {arg}");
        std::process::exit(2);
    }
    if std::env::args().any(|a| a == "--verify-completion") {
        if let Err(error) = completion::verify() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let lessons = match course::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        }
    };
    if std::env::args().any(|a| a == "--verify-art") {
        if let Err(error) = art::verify(&lessons) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let guides = match learning::load(&lessons) {
        Ok(guides) => guides,
        Err(error) => {
            eprintln!("{error}");
            return Ok(());
        }
    };
    if std::env::args().any(|a| a == "--verify-learning") {
        if let Err(error) = learning::verify(&lessons, &guides) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
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
        Box::new(move |cc| Ok(Box::new(App::new(&cc.egui_ctx, lessons, guides, size[0])))),
    )
}
