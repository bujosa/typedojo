//! Debounced suggestions, bound to an exact file/source/cursor snapshot.
use crate::{
    completion::{self, Entry, Request},
    design,
};
use egui::{Color32, Key, Modifiers, RichText};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
type Pending = (Request, mpsc::Receiver<Result<Vec<Entry>, String>>);
#[derive(Default)]
pub struct Suggestions {
    key: Option<Request>,
    pending: Option<Pending>,
    entries: Vec<Entry>,
    selected: usize,
    due: Option<Instant>,
    dismissed: bool,
    force: bool,
    clicked: bool,
    anchor: egui::Pos2,
    popup: Option<egui::Rect>,
    notice: Option<String>,
    explicit: bool,
    suppress_next: bool,
    scroll_selected: bool,
}
impl Suggestions {
    pub fn wants_focus(&self) -> bool {
        self.force
    }
    pub fn request(&mut self) {
        self.force = true;
    }
    fn clear(&mut self) {
        self.entries.clear();
        self.notice = None;
        self.due = None;
        self.dismissed = true;
        self.clicked = false;
    }
    pub fn before(
        &mut self,
        ui: &mut egui::Ui,
        key: &Request,
        widget: egui::Id,
        read_only: bool,
    ) -> Option<Entry> {
        let focused = ui.memory(|m| m.has_focus(widget));
        if read_only {
            self.clear();
            self.force = false;
            return None;
        }
        if focused && ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Space)) {
            self.force = true;
        }
        if self.force {
            ui.memory_mut(|m| m.request_focus(widget));
        }
        let outside = ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| self.popup.is_some_and(|r| !r.contains(p)))
        });
        if outside {
            self.clear();
        }
        if self.key.as_ref() != Some(key) {
            self.entries.clear();
            self.notice = None;
            self.clicked = false;
        }
        if self.key.as_ref() == Some(key) && !self.dismissed && (focused || self.clicked) {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                self.clear();
                return None;
            }
            if !self.entries.is_empty() {
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                    self.selected = (self.selected + 1) % self.entries.len();
                    self.scroll_selected = true;
                }
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                    self.selected = (self.selected + self.entries.len() - 1) % self.entries.len();
                    self.scroll_selected = true;
                }
                let accept = self.clicked
                    || ui.input_mut(|i| {
                        i.consume_key(Modifiers::NONE, Key::Tab)
                            || i.consume_key(Modifiers::NONE, Key::Enter)
                    });
                if accept {
                    let entry = self.entries.get(self.selected).cloned();
                    self.clear();
                    self.suppress_next = true;
                    return entry;
                }
            }
        }
        None
    }
    pub fn observe(
        &mut self,
        ctx: &egui::Context,
        key: Request,
        anchor: egui::Pos2,
        focused: bool,
        editable: bool,
    ) {
        self.anchor = anchor;
        if std::mem::take(&mut self.suppress_next) {
            self.key = Some(key);
            return;
        }
        if !editable || (!focused && !self.force) {
            self.clear();
            self.key = None;
            return;
        }
        let forced = std::mem::take(&mut self.force);
        if self.key.as_ref() != Some(&key) || forced {
            self.key = Some(key.clone());
            self.entries.clear();
            self.selected = 0;
            self.notice = None;
            self.dismissed = false;
            self.explicit = forced;
            let before: String = key.source.chars().take(key.cursor).collect();
            let prefix = before
                .chars()
                .rev()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .count();
            let eligible = forced || before.ends_with('.') || prefix >= 2;
            self.due = eligible.then(|| {
                Instant::now()
                    + if forced {
                        Duration::ZERO
                    } else {
                        Duration::from_millis(280)
                    }
            });
        }
        if let Some((_, rx)) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    if let Some((snapshot, _)) = self.pending.take()
                        && self.key.as_ref() == Some(&snapshot)
                        && !self.dismissed
                    {
                        self.due = None;
                        match result {
                            Ok(entries) => {
                                self.entries = entries;
                                self.scroll_selected = true;
                                if self.entries.is_empty() && self.explicit {
                                    self.notice = Some("No suggestions at this position".into());
                                }
                            }
                            Err(error) => {
                                if self.explicit {
                                    self.notice = Some(error);
                                }
                            }
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.due = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.pending.is_none()
            && !self.dismissed
            && self.due.is_some_and(|due| Instant::now() >= due)
        {
            let (tx, rx) = mpsc::channel();
            let request = key.clone();
            let repaint = ctx.clone();
            std::thread::spawn(move || {
                let result = completion::query(&request);
                let _ = tx.send(result);
                repaint.request_repaint();
            });
            self.pending = Some((key, rx));
            self.due = None;
        }
        if self.pending.is_some() || self.due.is_some() {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }
    pub fn popup(&mut self, ctx: &egui::Context, widget: egui::Id) {
        if self.dismissed
            || (self.entries.is_empty()
                && self.notice.is_none()
                && !(self.explicit && self.pending.is_some()))
        {
            self.popup = None;
            return;
        }
        let mut chosen = None;
        let response = egui::Area::new(widget.with("suggestions"))
            .order(egui::Order::Foreground)
            .fixed_pos(self.anchor)
            .constrain_to(ctx.content_rect().shrink(8.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(design::PANEL)
                    .stroke(egui::Stroke::new(1.0, design::BORDER))
                    .corner_radius(6.0)
                    .inner_margin(8.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 3],
                        blur: 12,
                        spread: 1,
                        color: Color32::from_black_alpha(85),
                    })
                    .show(ui, |ui| {
                        ui.set_width((ctx.content_rect().width() - 40.0).clamp(180.0, 350.0));
                        ui.label(RichText::new("Suggestions").size(11.0).color(design::MUTED));
                        if self.entries.is_empty() {
                            ui.label(
                                RichText::new(
                                    self.notice.as_deref().unwrap_or("Finding suggestions…"),
                                )
                                .size(12.0)
                                .color(design::MUTED),
                            );
                        }
                        egui::ScrollArea::vertical()
                            .max_height(160.0)
                            .id_salt(widget.with("suggest-list"))
                            .show(ui, |ui| {
                                for (index, entry) in self.entries.iter().enumerate() {
                                    let response = ui.add(
                                        egui::Button::new(
                                            RichText::new(format!(
                                                "{}   · {}",
                                                entry.label, entry.kind
                                            ))
                                            .monospace()
                                            .size(12.0),
                                        )
                                        .selected(index == self.selected)
                                        .min_size(egui::vec2(ui.available_width(), 25.0)),
                                    );
                                    if index == self.selected && self.scroll_selected {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                    if response.clicked() {
                                        chosen = Some(index);
                                    }
                                }
                            });
                        if let Some(entry) = self.entries.get(self.selected)
                            && !entry.detail.is_empty()
                        {
                            ui.separator();
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&entry.detail)
                                        .monospace()
                                        .size(11.0)
                                        .color(design::MUTED),
                                )
                                .wrap(),
                            );
                        }
                        ui.label(
                            RichText::new("↑↓ choose   Tab / Enter accept   Esc close")
                                .size(10.0)
                                .color(design::MUTED),
                        );
                    });
            });
        self.popup = Some(response.response.rect);
        self.scroll_selected = false;
        if let Some(index) = chosen {
            self.selected = index;
            self.clicked = true;
            ctx.memory_mut(|m| m.request_focus(widget));
            ctx.request_repaint();
        }
    }
}
