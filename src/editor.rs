//! Native code workbench. Syntax is presentation only; tsc remains the lab authority.
use crate::design;
use egui::{Color32, FontId, RichText, Stroke};
use std::sync::Arc;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const NAMES: &[&str] = &[
    "keyword",
    "type",
    "function",
    "property",
    "string",
    "comment",
    "number",
    "constant",
    "variable",
    "variable.parameter",
    "operator",
    "punctuation",
    "constructor",
    "escape",
];
const WELL: Color32 = Color32::from_rgb(12, 19, 29);
const GUTTER: Color32 = Color32::from_rgb(123, 142, 162);
fn color(index: usize) -> Color32 {
    match NAMES.get(index).copied().unwrap_or("") {
        "keyword" => Color32::from_rgb(202, 168, 240),
        "type" | "constructor" => Color32::from_rgb(110, 214, 190),
        "function" => Color32::from_rgb(226, 210, 151),
        "property" | "variable.parameter" => Color32::from_rgb(156, 211, 243),
        "string" => Color32::from_rgb(226, 173, 139),
        "comment" => Color32::from_rgb(142, 165, 153),
        "number" | "constant" => Color32::from_rgb(181, 211, 160),
        "operator" | "punctuation" => Color32::from_rgb(191, 204, 222),
        "escape" => Color32::from_rgb(238, 206, 140),
        _ => design::TEXT,
    }
}
fn append(job: &mut egui::text::LayoutJob, source: &str, color: Color32, size: f32) {
    job.append(
        source,
        0.0,
        egui::TextFormat {
            font_id: FontId::monospace(size),
            color,
            line_height: Some(size * 22.0 / 14.0),
            ..Default::default()
        },
    );
}
struct Syntax {
    config: Option<HighlightConfiguration>,
    highlighter: Highlighter,
}
impl Default for Syntax {
    fn default() -> Self {
        // TypeScript's queries add its type syntax to JavaScript's expression/string/comment queries.
        let query = format!(
            "{}\n{}",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_typescript::HIGHLIGHTS_QUERY
        );
        let config = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "typescript",
            &query,
            "",
            "",
        )
        .ok()
        .map(|mut c| {
            c.configure(NAMES);
            c
        });
        Self {
            config,
            highlighter: Highlighter::new(),
        }
    }
}
impl Syntax {
    fn job(&mut self, source: &str, size: f32) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;
        if let Some(config) = &self.config
            && let Ok(events) = self
                .highlighter
                .highlight(config, source.as_bytes(), None, |_| None)
        {
            let mut stack = Vec::new();
            let mut failed = false;
            for event in events {
                match event {
                    Ok(HighlightEvent::HighlightStart(h)) => stack.push(h.0),
                    Ok(HighlightEvent::HighlightEnd) => {
                        stack.pop();
                    }
                    Ok(HighlightEvent::Source { start, end }) => {
                        if let Some(part) = source.get(start..end) {
                            append(
                                &mut job,
                                part,
                                stack.last().map(|h| color(*h)).unwrap_or(design::TEXT),
                                size,
                            );
                        } else {
                            failed = true;
                            break;
                        }
                    }
                    Err(_) => {
                        failed = true;
                        break;
                    }
                }
            }
            if !failed && job.text == source {
                return job;
            }
        }
        let mut plain = egui::text::LayoutJob::default();
        plain.wrap.max_width = f32::INFINITY;
        append(&mut plain, source, design::TEXT, size);
        plain
    }
}
pub struct Options<'a> {
    pub expanded: bool,
    pub height: Option<f32>,
    pub read_only: bool,
    pub completion_enabled: bool,
    pub workbench: bool,
    pub name: &'a str,
    pub sibling: &'a str,
    pub breakpoints: Option<&'a mut std::collections::BTreeSet<usize>>,
    pub execution: Option<usize>,
    pub font_size: f32,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            expanded: false,
            height: None,
            read_only: false,
            completion_enabled: true,
            workbench: false,
            name: "answer.ts",
            sibling: "",
            breakpoints: None,
            execution: None,
            font_size: 14.0,
        }
    }
}
#[derive(Default)]
pub struct Editor {
    suggestions: crate::suggest::Suggestions,
    syntax: Syntax,
    source: String,
    pixels_per_point: f32,
    galley: Option<Arc<egui::Galley>>,
    cursor: (usize, usize),
    lesson: String,
    reveal: Option<usize>,
    cached_font_size: f32,
    find_open: bool,
    find_query: String,
    find_index: usize,
    find_focus: bool,
    selection: Option<(usize, usize)>,
}
impl Editor {
    pub fn request_suggestions(&mut self) {
        self.suggestions.request();
    }
    fn record_completion(ctx: &egui::Context, id: &str, before: &str, after: &str, cursor: usize) {
        let widget = egui::Id::new(("editor", id));
        let mut state = egui::TextEdit::load_state(ctx, widget).unwrap_or_default();
        let range = state.cursor.char_range().unwrap_or_else(|| {
            egui::text::CCursorRange::one(egui::text::CCursor::new(before.chars().count()))
        });
        let next = egui::text::CCursorRange::one(egui::text::CCursor::new(cursor));
        let mut undo = state.undoer();
        let time = ctx.input(|i| i.time);
        let old = (range, before.to_owned());
        let new = (next, after.to_owned());
        undo.feed_state(time, &old);
        undo.add_undo(&old);
        undo.feed_state(time, &new);
        undo.add_undo(&new);
        state.set_undoer(undo);
        state.cursor.set_char_range(Some(next));
        state.store(ctx, widget);
        ctx.memory_mut(|m| m.request_focus(widget));
    }
    /// Keep a formatter replacement separate from a recent paste or typing burst.
    pub fn record_format(ctx: &egui::Context, id: &str, before: &str, after: &str) {
        let widget_id = egui::Id::new(("editor", id));
        let mut state = egui::TextEdit::load_state(ctx, widget_id).unwrap_or_default();
        let before_range = state.cursor.char_range().unwrap_or_else(|| {
            egui::text::CCursorRange::one(egui::text::CCursor::new(before.chars().count()))
        });
        let mut after_range = before_range;
        let length = after.chars().count();
        after_range.primary.index = usize::from(after_range.primary.index).min(length).into();
        after_range.secondary.index = usize::from(after_range.secondary.index).min(length).into();
        let mut undoer = state.undoer();
        let time = ctx.input(|input| input.time);
        let before_state = (before_range, before.to_owned());
        let after_state = (after_range, after.to_owned());
        // feed_state clears any obsolete redo branch; add_undo makes both boundaries immediate.
        undoer.feed_state(time, &before_state);
        undoer.add_undo(&before_state);
        undoer.feed_state(time, &after_state);
        undoer.add_undo(&after_state);
        state.set_undoer(undoer);
        state.cursor.set_char_range(Some(after_range));
        state.store(ctx, widget_id);
    }
    fn layout(
        &mut self,
        ui: &egui::Ui,
        text: &dyn egui::TextBuffer,
        size: f32,
    ) -> Arc<egui::Galley> {
        if self.source == text.as_str()
            && self.pixels_per_point == ui.pixels_per_point()
            && self.cached_font_size == size
            && let Some(galley) = &self.galley
        {
            return galley.clone();
        }
        let job = self.syntax.job(text.as_str(), size);
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        self.source = text.as_str().to_owned();
        self.cached_font_size = size;
        self.pixels_per_point = ui.pixels_per_point();
        self.galley = Some(galley.clone());
        galley
    }
    pub fn reference(&mut self, ui: &mut egui::Ui, source: &str) {
        let galley = self.layout(ui, &source, 14.0);
        ui.add(egui::Label::new(galley).selectable(true));
    }
    pub fn open_find(&mut self) {
        self.find_open = true;
        self.find_focus = true;
    }
    pub fn find_bar(&mut self, ui: &mut egui::Ui, source: &str) {
        if !self.find_open {
            return;
        }
        let mut select = false;
        egui::Frame::new()
            .fill(design::PANEL)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.find_query)
                            .id_salt("find-in-file")
                            .return_key(None)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Find in file · match case")
                            .desired_width((ui.available_width() - 185.0).clamp(100.0, 360.0)),
                    );
                    if self.find_focus {
                        response.request_focus();
                        self.find_focus = false;
                    }
                    let total = if self.find_query.is_empty() {
                        0
                    } else {
                        source.match_indices(&self.find_query).count()
                    };
                    if response.changed() {
                        self.find_index = 0;
                        select = true;
                    }
                    self.find_index = self.find_index.min(total.saturating_sub(1));
                    ui.label(
                        RichText::new(if total == 0 {
                            "0 results".into()
                        } else {
                            format!("{} of {total}", self.find_index + 1)
                        })
                        .size(11.0)
                        .color(
                            if total == 0 && !self.find_query.is_empty() {
                                Color32::from_rgb(235, 199, 119)
                            } else {
                                design::MUTED
                            },
                        ),
                    );
                    if ui
                        .add_enabled(total > 0, egui::Button::new("↑"))
                        .on_hover_text("Previous match · Shift+Enter")
                        .clicked()
                    {
                        self.find_index = if self.find_index == 0 {
                            total.saturating_sub(1)
                        } else {
                            self.find_index - 1
                        };
                        select = true;
                    }
                    // Consume Shift+Enter first: egui allows extra Shift for an unmodified shortcut.
                    let previous = response.has_focus()
                        && ui
                            .input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter));
                    let enter = response.has_focus()
                        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    let next = ui
                        .add_enabled(total > 0, egui::Button::new("↓"))
                        .on_hover_text("Next match · Enter")
                        .clicked()
                        || enter;
                    if next && total > 0 {
                        self.find_index = (self.find_index + 1) % total;
                        select = true;
                    }
                    if previous && total > 0 {
                        self.find_index = if self.find_index == 0 {
                            total - 1
                        } else {
                            self.find_index - 1
                        };
                        select = true;
                    }
                    if ui
                        .button("×")
                        .on_hover_text("Close find · Escape")
                        .clicked()
                        || ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
                    {
                        self.find_open = false;
                    }
                });
            });
        if select
            && !self.find_query.is_empty()
            && let Some((start, matched)) =
                source.match_indices(&self.find_query).nth(self.find_index)
        {
            let a = source[..start].chars().count();
            let b = a + matched.chars().count();
            self.selection = Some((a, b));
            self.reveal = Some(source[..start].bytes().filter(|b| *b == b'\n').count() + 1);
        }
    }
    pub fn reveal_line(&mut self, line: usize) {
        self.reveal = Some(line.max(1));
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        source: &mut String,
        id: &str,
        mut options: Options<'_>,
    ) -> bool {
        if self.lesson != id {
            self.cursor = (1, 1);
            self.lesson = id.to_owned();
        }
        let size = options.font_size.clamp(11.0, 22.0);
        let row_height = size * 22.0 / 14.0;
        let widget_id = egui::Id::new(("editor", id));
        let selection = self.selection.take();
        let reveal = self.reveal.take();
        if let Some(line) = reveal {
            let index = if line <= 1 {
                0
            } else {
                source
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| *c == '\n')
                    .nth(line - 2)
                    .map(|(i, _)| i + 1)
                    .unwrap_or(source.chars().count())
            };
            let mut state = egui::TextEdit::load_state(ui.ctx(), widget_id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(if let Some((a, b)) = selection {
                    egui::text::CCursorRange::two(
                        egui::text::CCursor::new(a),
                        egui::text::CCursor::new(b),
                    )
                } else {
                    egui::text::CCursorRange::one(egui::text::CCursor::new(index))
                }));
            state.store(ui.ctx(), widget_id);
            if selection.is_none() {
                ui.memory_mut(|m| m.request_focus(widget_id));
            }
        }
        let mut completed = false;
        if options.workbench {
            let cursor = egui::TextEdit::load_state(ui.ctx(), widget_id)
                .and_then(|s| s.cursor.char_range())
                .map(|r| usize::from(r.primary.index))
                .unwrap_or(0);
            let request = crate::completion::Request {
                id: id.to_owned(),
                file: options.name.to_owned(),
                source: source.clone(),
                sibling: options.sibling.to_owned(),
                cursor,
            };
            if let Some(entry) = self.suggestions.before(
                ui,
                &request,
                widget_id,
                options.read_only || !options.completion_enabled,
            ) && let Ok((after, cursor)) = entry.apply(&request)
            {
                Self::record_completion(ui.ctx(), id, source, &after, cursor);
                *source = after;
                completed = true;
            }
        }
        if !options.read_only && ui.memory(|m| m.has_focus(widget_id)) {
            let range =
                egui::TextEdit::load_state(ui.ctx(), widget_id).and_then(|s| s.cursor.char_range());
            if let Some(range) = range {
                let before: String = source.chars().take(range.primary.index.into()).collect();
                let line = before.rsplit('\n').next().unwrap_or("");
                let mut indent: String = line
                    .chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .collect();
                if line.trim_end().ends_with(['{', '[', '(']) {
                    indent.push_str("    ");
                }
                let insertion = format!("\n{indent}");
                ui.input_mut(|i| {
                    for event in &mut i.events {
                        if let egui::Event::Key {
                            key: egui::Key::Enter,
                            pressed: true,
                            modifiers,
                            ..
                        } = event
                            && !modifiers.command
                            && !modifiers.ctrl
                            && !modifiers.alt
                        {
                            *event = egui::Event::Text(insertion.clone());
                        }
                    }
                });
            }
        }
        let width = ui.available_width();
        let mut changed = completed;
        egui::Frame::new()
            .fill(WELL)
            .stroke(if options.workbench {
                Stroke::NONE
            } else {
                Stroke::new(1.0, design::BORDER)
            })
            .corner_radius(if options.workbench { 0.0 } else { 8.0 })
            .show(ui, |ui| {
                ui.set_min_width((width - 2.0).max(0.0));
                if !options.workbench {
                    egui::Frame::new()
                        .fill(design::PANEL)
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.set_min_width((width - 30.0).max(0.0));
                            ui.horizontal(|ui| {
                                let (dot, _) = ui.allocate_exact_size(
                                    egui::vec2(7.0, 7.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter()
                                    .circle_filled(dot.center(), 3.0, design::ACCENT);
                                ui.label(
                                    RichText::new(options.name)
                                        .monospace()
                                        .size(13.0)
                                        .color(design::TEXT),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new("TypeScript")
                                                .size(11.5)
                                                .color(design::MUTED),
                                        );
                                    },
                                );
                            });
                        });
                }
                let viewport_width = (width - 20.0).max(100.0);
                let text_width = (source.lines().map(|l| l.chars().count()).max().unwrap_or(0)
                    as f32
                    * size
                    * 8.5
                    / 14.0
                    + 24.0)
                    .max(viewport_width - 60.0);
                let editor_height = options.height.unwrap_or_else(|| {
                    if options.expanded {
                        (ui.ctx().content_rect().height() - 430.0).clamp(260.0, 620.0)
                    } else {
                        (ui.ctx().content_rect().height() - 600.0).clamp(180.0, 320.0)
                    }
                });
                let gutter_left = ui.cursor().min.x;
                let mut scroll = egui::ScrollArea::both();
                if let Some(line) = reveal {
                    scroll = scroll.vertical_scroll_offset(
                        (line.saturating_sub(1) as f32 * row_height - row_height * 2.0).max(0.0),
                    );
                }
                scroll
                    .id_salt(("code-scroll", id))
                    .max_height(editor_height)
                    .min_scrolled_height(editor_height.min(140.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add_space(10.0);
                        ui.horizontal_top(|ui| {
                            let start = ui.cursor().min;
                            ui.add_space(52.0);
                            let highlight = ui.painter().add(egui::Shape::Noop);
                            let mut layouter =
                                |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
                                    self.layout(ui, text, size)
                                };
                            let previous_clip = ui.clip_rect();
                            ui.set_clip_rect(previous_clip.intersect(egui::Rect::from_min_max(
                                egui::pos2(gutter_left + 48.0, previous_clip.top()),
                                previous_clip.max,
                            )));
                            let output = egui::TextEdit::multiline(source)
                                .id(egui::Id::new(("editor", id)))
                                .code_editor()
                                .interactive(!options.read_only)
                                .frame(egui::Frame::new())
                                .margin(egui::Margin::symmetric(6, 0))
                                .desired_width(text_width)
                                .desired_rows(if options.workbench {
                                    (editor_height / row_height).max(5.0) as usize
                                } else {
                                    14
                                })
                                .font(FontId::monospace(size))
                                .layouter(&mut layouter)
                                .show(ui);
                            ui.set_clip_rect(previous_clip);
                            if self.suggestions.wants_focus() {
                                output.response.request_focus();
                            }
                            changed |= output.response.changed();
                            if options.workbench
                                && let Some(range) = output.cursor_range
                            {
                                let cursor = usize::from(range.primary.index);
                                let rect = output
                                    .galley
                                    .pos_from_cursor(range.primary)
                                    .translate(output.galley_pos.to_vec2());
                                let request = crate::completion::Request {
                                    id: id.to_owned(),
                                    file: options.name.to_owned(),
                                    source: source.clone(),
                                    sibling: options.sibling.to_owned(),
                                    cursor,
                                };
                                self.suggestions.observe(
                                    ui.ctx(),
                                    request,
                                    rect.left_bottom() + egui::vec2(0.0, 4.0),
                                    output.response.has_focus(),
                                    !options.read_only
                                        && options.completion_enabled
                                        && range.is_empty(),
                                );
                            }
                            let active = output
                                .cursor_range
                                .map(|r| output.galley.layout_from_cursor(r.primary).row);
                            if let Some(range) = output.cursor_range {
                                let before: String =
                                    source.chars().take(range.primary.index.into()).collect();
                                self.cursor = (
                                    before.chars().filter(|c| *c == '\n').count() + 1,
                                    before
                                        .rsplit('\n')
                                        .next()
                                        .map(|s| s.chars().count() + 1)
                                        .unwrap_or(1),
                                );
                            }
                            if self.find_open && !self.find_query.is_empty() {
                                let positions: Vec<usize> =
                                    source.char_indices().map(|(byte, _)| byte).collect();
                                let matches: Vec<_> =
                                    source.match_indices(&self.find_query).collect();
                                let painter = ui.painter().with_clip_rect(previous_clip.intersect(
                                    egui::Rect::from_min_max(
                                        egui::pos2(gutter_left + 48.0, previous_clip.top()),
                                        previous_clip.max,
                                    ),
                                ));
                                for (index, (byte, matched)) in matches
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| *i < 100 || *i == self.find_index)
                                {
                                    let a =
                                        positions.binary_search(byte).unwrap_or(positions.len());
                                    let b = positions
                                        .binary_search(&(byte + matched.len()))
                                        .unwrap_or(positions.len());
                                    let start_rect = output
                                        .galley
                                        .pos_from_cursor(egui::text::CCursor::new(a))
                                        .translate(output.galley_pos.to_vec2());
                                    let end_rect = output
                                        .galley
                                        .pos_from_cursor(egui::text::CCursor::new(b))
                                        .translate(output.galley_pos.to_vec2());
                                    let rect = egui::Rect::from_min_max(
                                        start_rect.min,
                                        egui::pos2(
                                            end_rect.left().max(start_rect.left() + 2.0),
                                            start_rect.bottom(),
                                        ),
                                    );
                                    let selected = index == self.find_index;
                                    painter.rect_filled(
                                        rect,
                                        1.0,
                                        if selected {
                                            Color32::from_rgba_unmultiplied(235, 199, 119, 45)
                                        } else {
                                            Color32::from_rgba_unmultiplied(125, 187, 244, 25)
                                        },
                                    );
                                    if selected {
                                        painter.rect_stroke(
                                            rect,
                                            1.0,
                                            Stroke::new(1.0, Color32::from_rgb(235, 199, 119)),
                                            egui::StrokeKind::Inside,
                                        );
                                    }
                                    if selected && !output.response.has_focus() {
                                        self.cursor = (
                                            source[..*byte].bytes().filter(|b| *b == b'\n').count()
                                                + 1,
                                            source[..*byte]
                                                .rsplit('\n')
                                                .next()
                                                .unwrap_or("")
                                                .chars()
                                                .count()
                                                + 1,
                                        );
                                    }
                                }
                            }
                            let highlighted = options
                                .execution
                                .map(|line| line.saturating_sub(1))
                                .or_else(|| {
                                    if output.response.has_focus() {
                                        active
                                    } else {
                                        None
                                    }
                                });
                            if let Some(index) = highlighted
                                && let Some(row) = output.galley.rows.get(index)
                            {
                                let y = output.galley_pos.y + row.pos.y;
                                let rect = egui::Rect::from_min_size(
                                    egui::pos2(start.x, y),
                                    egui::vec2(output.response.rect.right() - start.x, row_height),
                                );
                                ui.painter().set(
                                    highlight,
                                    egui::Shape::rect_filled(
                                        rect,
                                        0.0,
                                        if options.execution.is_some() {
                                            Color32::from_rgb(69, 57, 29)
                                        } else {
                                            Color32::from_rgb(23, 36, 52)
                                        },
                                    ),
                                );
                            }
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(gutter_left, start.y),
                                    egui::vec2(48.0, output.response.rect.height()),
                                ),
                                0.0,
                                WELL,
                            );
                            for (index, row) in output.galley.rows.iter().enumerate() {
                                let y = output.galley_pos.y + row.pos.y + row.rect().height() / 2.0;
                                if let Some(points) = options.breakpoints.as_deref_mut() {
                                    let line = index + 1;
                                    let rect = egui::Rect::from_center_size(
                                        egui::pos2(gutter_left + 10.0, y),
                                        egui::vec2(18.0, row_height),
                                    );
                                    if ui.is_rect_visible(rect) {
                                        let response = ui.interact(
                                            rect,
                                            widget_id.with(("breakpoint", line)),
                                            egui::Sense::click(),
                                        );
                                        if response.clicked() && !options.read_only {
                                            if !points.remove(&line) {
                                                points.insert(line);
                                            }
                                            changed = true;
                                        }
                                        response.on_hover_text(if options.read_only {
                                            "Stop the session to change breakpoints"
                                        } else {
                                            "Toggle breakpoint"
                                        });
                                        if points.contains(&line) {
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                4.0,
                                                Color32::from_rgb(231, 104, 102),
                                            );
                                        }
                                    }
                                }
                                if options.execution == Some(index + 1) {
                                    ui.painter().text(
                                        egui::pos2(gutter_left + 19.0, y),
                                        egui::Align2::CENTER_CENTER,
                                        ">",
                                        FontId::monospace(13.0),
                                        Color32::from_rgb(242, 204, 112),
                                    );
                                }
                                ui.painter().text(
                                    egui::pos2(gutter_left + 38.0, y),
                                    egui::Align2::RIGHT_CENTER,
                                    (index + 1).to_string(),
                                    FontId::monospace(12.0),
                                    if active == Some(index) {
                                        design::TEXT
                                    } else {
                                        GUTTER
                                    },
                                );
                            }
                            ui.painter().line_segment(
                                [
                                    egui::pos2(gutter_left + 46.0, start.y),
                                    egui::pos2(gutter_left + 46.0, output.response.rect.bottom()),
                                ],
                                Stroke::new(1.0, design::BORDER),
                            );
                        });
                        ui.add_space(10.0);
                    });
                egui::Frame::new()
                    .fill(design::PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 7))
                    .show(ui, |ui| {
                        ui.set_min_width((width - 26.0).max(0.0));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Ln {}, Col {}",
                                    self.cursor.0.max(1),
                                    self.cursor.1.max(1)
                                ))
                                .monospace()
                                .size(11.5)
                                .color(design::MUTED),
                            );
                            ui.label(
                                RichText::new("UTF-8  ·  strict  ·  ES2022")
                                    .size(11.5)
                                    .color(design::MUTED),
                            );
                            if self.syntax.config.is_none() {
                                ui.colored_label(design::ACCENT, "Plain text · syntax unavailable");
                            }
                        });
                    });
            });
        if options.workbench {
            self.suggestions.popup(ui.ctx(), widget_id);
        }
        changed
    }
}
