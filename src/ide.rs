//! IDE presentation and session state; execution lives in engine, completion lives in App.
use crate::{
    commands::{Action, Command, Palette},
    course::Lesson,
    design, editor, engine,
    store::Draft,
};
use egui::{Color32, RichText, Stroke};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
const AMBER: Color32 = Color32::from_rgb(235, 199, 119);
const RED: Color32 = Color32::from_rgb(240, 139, 135);
const GREEN: Color32 = Color32::from_rgb(116, 212, 157);
#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Compiling,
    Running,
    Paused,
    Stopping,
    Finished,
    Failed,
}
impl State {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Compiling => "Compiling",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Stopping => "Stopping",
            Self::Finished => "Finished",
            Self::Failed => "Failed",
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum LogFilter {
    All,
    Output,
    Warnings,
    Errors,
}
struct Log {
    at: Duration,
    level: String,
    text: String,
}
pub struct Context<'a> {
    pub guided: bool,
    pub formatting: bool,
    pub format_notice: Option<(&'a str, bool)>,
}
#[derive(Default)]
pub struct Response {
    pub changed: bool,
    pub check: bool,
    pub format: Option<String>,
    pub solution: bool,
    pub reset: bool,
}
pub struct Workbench {
    lesson: String,
    editor: editor::Editor,
    session: Option<engine::Session>,
    state: State,
    file: String,
    debug_view: bool,
    sidebar: bool,
    layout_set: bool,
    panel: usize,
    logs: VecDeque<Log>,
    problems: String,
    frames: Vec<engine::Frame>,
    frame: usize,
    expression: String,
    task: bool,
    started: Option<Instant>,
    elapsed: Duration,
    debug: bool,
    breakpoint_line: String,
    console_height: f32,
    snapshot: Option<(String, String)>,
    palette: Palette,
    font_size: f32,
    log_query: String,
    log_filter: LogFilter,
    follow: bool,
    copied: Option<Instant>,
    guided: bool,
    formatting: bool,
    task_docked: bool,
}
impl Default for Workbench {
    fn default() -> Self {
        Self {
            lesson: String::new(),
            editor: editor::Editor::default(),
            session: None,
            state: State::Idle,
            file: "answer.ts".into(),
            debug_view: false,
            sidebar: true,
            layout_set: false,
            panel: 0,
            logs: VecDeque::new(),
            problems: String::new(),
            frames: Vec::new(),
            frame: 0,
            expression: String::new(),
            task: false,
            started: None,
            elapsed: Duration::ZERO,
            debug: false,
            breakpoint_line: String::new(),
            console_height: 190.0,
            snapshot: None,
            palette: Palette::default(),
            font_size: 14.0,
            log_query: String::new(),
            log_filter: LogFilter::All,
            follow: true,
            copied: None,
            guided: false,
            formatting: false,
            task_docked: true,
        }
    }
}
impl Workbench {
    pub fn active_file(&self) -> &str {
        &self.file
    }
    pub fn stop(&mut self) {
        if self.session.is_some() {
            self.command("stop");
        }
    }
    pub fn allows_check(&self) -> bool {
        self.session.is_none() && !self.task && !self.palette.is_open()
    }
    pub fn select(&mut self, id: &str) {
        if self.lesson != id {
            let appearance = (
                self.font_size,
                self.sidebar,
                self.layout_set,
                self.console_height,
                self.follow,
            );
            *self = Self::default();
            (
                self.font_size,
                self.sidebar,
                self.layout_set,
                self.console_height,
                self.follow,
            ) = appearance;
            self.lesson = id.into();
        }
    }
    fn log(&mut self, level: &str, text: String) {
        if self.logs.len() >= 500 {
            self.logs.pop_front();
        }
        self.logs.push_back(Log {
            at: self.started.map(|s| s.elapsed()).unwrap_or(self.elapsed),
            level: level.into(),
            text,
        });
    }
    pub fn poll(&mut self, ctx: &egui::Context) {
        let events: Vec<_> = self
            .session
            .as_ref()
            .map(|s| s.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                engine::Event::Running => {
                    self.state = State::Running;
                    self.frames.clear();
                }
                engine::Event::Pausing => self.state = State::Paused,
                engine::Event::Paused { reason, frames } => {
                    self.state = State::Paused;
                    self.frame = 0;
                    self.frames = frames;
                    self.debug_view = true;
                    self.sidebar = true;
                    if let Some(frame) = self.frames.first() {
                        self.file = frame.file.clone();
                        self.editor.reveal_line(frame.line);
                    }
                    self.log("system", format!("Paused · {reason}"));
                }
                engine::Event::Log { level, text } => self.log(&level, text),
                engine::Event::Evaluation { text } => self.log("evaluation", text),
                engine::Event::CommandError { text } => self.log("error", text),
                engine::Event::Breakpoint {
                    file,
                    requested,
                    line,
                } => self.log(
                    "system",
                    if requested == line {
                        format!("Breakpoint bound · {file}:{line}")
                    } else {
                        format!("Breakpoint moved to executable code · {file}:{requested} → {line}")
                    },
                ),
                engine::Event::Problems { text } => {
                    self.problems = text;
                    self.panel = 1;
                }
                engine::Event::Finished { success } => {
                    self.state = if success {
                        State::Finished
                    } else {
                        State::Failed
                    };
                    self.finish();
                    self.log(
                        if success { "system" } else { "error" },
                        if success {
                            "Process exited successfully. Run does not mark a lab as passed."
                        } else {
                            "Execution failed. Review Console and Problems."
                        }
                        .into(),
                    );
                }
                engine::Event::Failure { text } => {
                    self.state = State::Failed;
                    self.finish();
                    self.log("error", text);
                    self.panel = 0;
                }
                engine::Event::Stopped => {
                    self.state = State::Idle;
                    self.finish();
                    self.log("system", "Session stopped.".into());
                }
            }
        }
        if self.session.is_some() {
            ctx.request_repaint_after(Duration::from_millis(30));
        }
    }
    fn finish(&mut self) {
        self.elapsed = self.started.map(|s| s.elapsed()).unwrap_or_default();
        self.started = None;
        self.session = None;
        self.frames.clear();
    }
    fn start(&mut self, draft: &Draft, debug: bool) {
        if self.session.is_some() {
            return;
        }
        self.logs.clear();
        self.problems.clear();
        self.frames.clear();
        self.panel = 0;
        self.snapshot = Some((
            draft.source.clone(),
            draft
                .playground
                .as_deref()
                .unwrap_or(engine::PLAYGROUND)
                .into(),
        ));
        self.state = State::Compiling;
        self.started = Some(Instant::now());
        self.debug = debug;
        self.log(
            "system",
            format!(
                "{} · strict TypeScript → Node.js",
                if debug { "Debug" } else { "Run" }
            ),
        );
        self.session = Some(engine::Session::start(engine::Request {
            answer: draft.source.clone(),
            playground: draft
                .playground
                .as_deref()
                .unwrap_or(engine::PLAYGROUND)
                .into(),
            debug,
            breakpoints: draft.breakpoints.clone(),
        }));
    }
    fn command(&mut self, action: &str) {
        if let Some(session) = &self.session {
            session.command(action);
        }
        if action == "stop" {
            self.state = State::Stopping;
        }
    }
    fn state_indicator(&self, ui: &mut egui::Ui) {
        let color = match self.state {
            State::Paused => AMBER,
            State::Failed => RED,
            State::Finished => GREEN,
            _ => design::MUTED,
        };
        if matches!(
            self.state,
            State::Compiling | State::Running | State::Stopping
        ) {
            ui.add(egui::Spinner::new().size(12.0));
        } else {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 2.5, color);
        }
        ui.label(RichText::new(self.state.label()).size(11.0).color(color))
            .on_hover_text("Run / Debug status. Lab completion is shown separately.");
    }
    fn commands(&self, checking: bool) -> Vec<Command> {
        let idle = self.session.is_none() && !checking && !self.formatting;
        let paused = self.state == State::Paused;
        vec![
            Command {
                action: Action::Format,
                name: "Format active file (Prettier)",
                shortcut: "Cmd+Shift+I",
                enabled: idle,
            },
            Command {
                action: Action::Solution,
                name: "Show worked solution",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Reset,
                name: "Reset this draft…",
                shortcut: "",
                enabled: idle,
            },
            Command {
                action: Action::Run,
                name: "Run playground",
                shortcut: "Cmd+R",
                enabled: idle,
            },
            Command {
                action: Action::Debug,
                name: "Start debugging",
                shortcut: "F5",
                enabled: idle,
            },
            Command {
                action: Action::Continue,
                name: "Continue debugging",
                shortcut: "F5",
                enabled: paused,
            },
            Command {
                action: Action::Stop,
                name: "Stop execution",
                shortcut: "",
                enabled: self.session.is_some(),
            },
            Command {
                action: Action::Over,
                name: "Step over",
                shortcut: "F10",
                enabled: paused,
            },
            Command {
                action: Action::Into,
                name: "Step into",
                shortcut: "F11",
                enabled: paused,
            },
            Command {
                action: Action::Out,
                name: "Step out",
                shortcut: "Shift+F11",
                enabled: paused,
            },
            Command {
                action: Action::Check,
                name: if self.guided {
                    "Check guided practice"
                } else {
                    "Check lab"
                },
                shortcut: "Cmd+Enter",
                enabled: idle,
            },
            Command {
                action: Action::Find,
                name: "Find in file",
                shortcut: "Cmd+F",
                enabled: true,
            },
            Command {
                action: Action::Answer,
                name: "Open file: answer.ts",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Playground,
                name: "Open file: playground.ts",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Task,
                name: "Show lesson task",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Sidebar,
                name: "Toggle sidebar",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Console,
                name: "Show console",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Problems,
                name: "Show problems",
                shortcut: "",
                enabled: true,
            },
            Command {
                action: Action::Larger,
                name: "Increase editor text size",
                shortcut: "",
                enabled: self.font_size < 22.0,
            },
            Command {
                action: Action::Smaller,
                name: "Decrease editor text size",
                shortcut: "",
                enabled: self.font_size > 11.0,
            },
        ]
    }
    fn apply(&mut self, action: Action, draft: &Draft, checking: bool) -> bool {
        let idle = self.session.is_none() && !checking && !self.formatting;
        match action {
            Action::Run if idle => self.start(draft, false),
            Action::Debug if idle => self.start(draft, true),
            Action::Stop => self.stop(),
            Action::Continue => self.command("continue"),
            Action::Over => self.command("over"),
            Action::Into => self.command("into"),
            Action::Out => self.command("out"),
            Action::Check if idle => {
                self.panel = 2;
                return true;
            }
            Action::Find => self.editor.open_find(),
            Action::Task => self.task = true,
            Action::Answer => self.file = "answer.ts".into(),
            Action::Playground => self.file = "playground.ts".into(),
            Action::Sidebar => self.sidebar = !self.sidebar,
            Action::Console => self.panel = 0,
            Action::Problems => self.panel = 1,
            Action::Larger => self.font_size = (self.font_size + 1.0).min(22.0),
            Action::Smaller => self.font_size = (self.font_size - 1.0).max(11.0),
            _ => {}
        }
        false
    }
    fn matches_log(&self, log: &Log) -> bool {
        let category = match self.log_filter {
            LogFilter::All => true,
            LogFilter::Output => {
                !matches!(log.level.as_str(), "system" | "error" | "warning" | "warn")
            }
            LogFilter::Warnings => matches!(log.level.as_str(), "warning" | "warn"),
            LogFilter::Errors => log.level == "error",
        };
        category
            && log
                .text
                .to_lowercase()
                .contains(&self.log_query.to_lowercase())
    }
    fn output(
        &mut self,
        ui: &mut egui::Ui,
        draft: &Draft,
        checking: bool,
        check_text: Option<&str>,
        paused: bool,
        max_height: f32,
    ) {
        egui::Panel::bottom("ide-output-modern").resizable(false).exact_size(self.console_height)
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(12,19,29)).inner_margin(egui::Margin::symmetric(10,6)))
            .show(ui,|ui|{
                ui.horizontal(|ui|{
                    ui.selectable_value(&mut self.panel,0,RichText::new("Console").size(12.0));
                    let count=self.problems.lines().filter(|l|l.contains("error TS")).count();
                    ui.selectable_value(&mut self.panel,1,RichText::new(format!("Problems{}",if count>0 {format!(" ({count})")}else{String::new()})).size(12.0));
                    ui.selectable_value(&mut self.panel,2,RichText::new(if self.guided {"Practice result"}else{"Lab result"}).size(12.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                        if self.panel==0 {
                            if ui.small_button("Clear").clicked(){self.logs.clear();}
                            let copied=self.copied.is_some_and(|at|at.elapsed()<Duration::from_secs(2));
                            if ui.small_button(if copied {"Copied"}else{"Copy"}).on_hover_text("Copy the visible logs").clicked(){
                                ui.ctx().copy_text(self.logs.iter().filter(|log|self.matches_log(log)).map(|log|format!("[{:.2}s] [{}] {}",log.at.as_secs_f32(),log.level,log.text)).collect::<Vec<_>>().join("\n"));
                                self.copied=Some(Instant::now());ui.ctx().request_repaint_after(Duration::from_secs(2));
                            }
                        }
                        if ui.small_button(if self.console_height>230.0 {"Restore"}else{"Expand"}).clicked(){self.console_height=if self.console_height>230.0 {190.0}else{max_height};}
                        let elapsed=self.started.map(|s|s.elapsed()).unwrap_or(self.elapsed);
                        if elapsed.as_millis()>0&&ui.available_width()>70.0{ui.label(RichText::new(format!("{:.1}s",elapsed.as_secs_f32())).monospace().size(11.0).color(design::MUTED));}
                    });
                });
                ui.separator();
                if self.panel<2&&self.snapshot.as_ref().is_some_and(|(answer,playground)|answer!=&draft.source||playground!=draft.playground.as_deref().unwrap_or(engine::PLAYGROUND)){
                    ui.label(RichText::new("Code changed · run again for current results").size(11.0).color(AMBER));
                }
                if self.panel==0 {
                    ui.horizontal_wrapped(|ui|{
                        ui.spacing_mut().item_spacing.x=6.0;
                        ui.add(egui::TextEdit::singleline(&mut self.log_query).id_salt("filter-logs").hint_text("Filter logs…").desired_width((ui.available_width()*0.25).clamp(100.0,220.0)));
                        for (filter,label) in [(LogFilter::All,"All"),(LogFilter::Output,"Output"),(LogFilter::Warnings,"Warnings"),(LogFilter::Errors,"Errors")]{ui.selectable_value(&mut self.log_filter,filter,RichText::new(label).size(11.0));}
                        ui.toggle_value(&mut self.follow,RichText::new("Follow").size(11.0)).on_hover_text("Keep new messages visible. Scrolling up pauses following until you return to the bottom.");
                        let shown=self.logs.iter().filter(|log|self.matches_log(log)).count();
                        ui.label(RichText::new(format!("{shown}/{}",self.logs.len())).monospace().size(10.5).color(design::MUTED));
                    });
                    if paused {
                        ui.horizontal(|ui|{
                            ui.label(RichText::new(">").monospace().color(AMBER));
                            let input=ui.add(egui::TextEdit::singleline(&mut self.expression).font(egui::TextStyle::Monospace).hint_text("Expression in selected frame · read only").desired_width((ui.available_width()-90.0).max(100.0)));
                            if (ui.button("Evaluate").clicked()||(input.lost_focus()&&ui.input(|i|i.key_pressed(egui::Key::Enter))))&&!self.expression.trim().is_empty()&&let Some(frame)=self.frames.get(self.frame).map(|f|f.id.clone()){
                                self.log("system",format!("> {}",self.expression));if let Some(session)=&self.session{session.evaluate(&frame,&self.expression);}
                            }
                        });
                    }
                    egui::ScrollArea::both().id_salt("ide-log-list").stick_to_bottom(self.follow).auto_shrink([false,false]).show(ui,|ui|{
                        if self.logs.is_empty(){ui.add_space(8.0);ui.label(RichText::new("Your console is ready").size(13.0));ui.label(RichText::new("Run playground.ts to inspect values with console.log.").size(12.0).color(design::MUTED));}
                        else if !self.logs.iter().any(|log|self.matches_log(log)){
                            ui.add_space(8.0);ui.label(RichText::new("No matching logs").size(12.0).color(design::MUTED));
                            if ui.small_button("Clear filters").clicked(){self.log_query.clear();self.log_filter=LogFilter::All;}
                        }
                        for log in self.logs.iter().filter(|log|self.matches_log(log)){
                            let color=match log.level.as_str(){"error"=>RED,"warning"|"warn"=>AMBER,"system"=>design::MUTED,"evaluation"=>GREEN,_=>design::TEXT};
                            ui.add(egui::Label::new(RichText::new(format!("{:>6.2}s  {:<5}  {}",log.at.as_secs_f32(),match log.level.as_str(){"error"=>"ERROR","warning"|"warn"=>"WARN","system"=>"SYS","evaluation"=>"EVAL",_=>"LOG"},log.text)).monospace().size(12.0).color(color)).selectable(true));
                        }
                    });
                }else{
                    let text=if self.panel==1{if self.problems.is_empty(){"No compiler diagnostics. Run or Debug to compile the workspace."}else{&self.problems}}else{check_text.unwrap_or(if checking{"Checking the current answer…"}else if draft.passed(){if self.guided {"This exact answer passed guided practice. Continue to the independent lab when ready."}else{"This exact answer passed the lab."}}else{if self.guided {"Check practice validates this smaller exercise. Its result is separate from the lab."}else{"Check lab validates your answer. Run executes the playground."}})}.to_string();
                    egui::ScrollArea::both().id_salt("ide-diagnostic-list").auto_shrink([false,false]).show(ui,|ui|{
                        for line in text.lines(){
                            let target=["answer.ts","playground.ts"].iter().find_map(|file|line.strip_prefix(&format!("{file}("))?.split_once(',')?.0.parse::<usize>().ok().map(|n|(*file,n)));
                            if let Some((file,line_number))=target{if ui.add(egui::Button::new(RichText::new(line).monospace().size(12.0).color(design::ACCENT)).frame(false)).clicked(){self.file=file.into();self.editor.reveal_line(line_number);}}
                            else{ui.label(RichText::new(line).monospace().size(12.0).color(design::MUTED));}
                        }
                    });
                }
            });
    }
    fn task_contents(&self, ui: &mut egui::Ui, lesson: &Lesson, busy: bool) -> (bool, bool) {
        let mut solution = false;
        let mut reset = false;
        egui::ScrollArea::vertical().id_salt("task-copy").show(ui,|ui|{
            design::eyebrow(ui,if self.guided {"GUIDED PRACTICE"}else{"INDEPENDENT LAB"},design::ACCENT);ui.add_space(10.0);
            ui.label(RichText::new(&lesson.title).font(design::display(16.0)));ui.add_space(12.0);
            ui.label(RichText::new(&lesson.task).size(13.5).line_height(Some(21.0)));ui.add_space(14.0);
            egui::CollapsingHeader::new("Need a hint?").show(ui,|ui|{ui.label(RichText::new(&lesson.hint).size(13.0));});
            egui::CollapsingHeader::new(if self.guided {"Review the reasoning"}else{"Full rules and context"}).default_open(lesson.id.starts_with("project-")).show(ui,|ui|{for paragraph in lesson.concept.split("\n\n"){ui.label(RichText::new(paragraph).size(12.5).line_height(Some(20.0)));ui.add_space(8.0);}});
            ui.add_space(14.0);if ui.button("Worked solution").clicked(){solution=true;}
            ui.label(RichText::new("A worked solution explains an approach; viewing it does not complete the exercise.").size(11.0).color(design::MUTED));
            if ui.add_enabled(!busy,egui::Button::new("Reset draft…")).clicked(){reset=true;}
        });
        (solution, reset)
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        lesson: &Lesson,
        draft: &mut Draft,
        checking: bool,
        check_text: Option<&str>,
        context: Context<'_>,
    ) -> Response {
        self.select(&lesson.id);
        self.guided = context.guided;
        self.formatting = context.formatting;
        let ctx = ui.ctx().clone();
        let mut changed = false;
        let mut check = false;
        let mut format = None;
        let mut solution = false;
        let mut reset = false;
        let shortcut = egui::Modifiers {
            command: true,
            shift: true,
            ..Default::default()
        };
        if !self.task && ctx.input_mut(|i| i.consume_key(shortcut, egui::Key::P)) {
            self.palette.open();
        }
        if !self.task
            && !self.palette.is_open()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
        {
            self.editor.open_find();
        }
        let active = self.session.is_some();
        let paused = self.state == State::Paused;
        let compact = ui.available_width() < 1000.0;
        if !self.layout_set {
            self.sidebar = false;
            self.layout_set = true;
        }
        ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
        let visuals = ui.visuals_mut();
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = egui::CornerRadius::same(4);
        }
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(12.0));
        // Focus is a workbench, not an enlarged lesson page: fixed commands, flexible editor,
        // independently resized output, optional navigation/inspection.
        egui::Panel::top("ide-commands")
            .frame(
                egui::Frame::new()
                    .fill(design::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 7)),
            )
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 5.0);
                    if ui
                        .add_enabled(
                            !active && !checking && !context.formatting,
                            egui::Button::new(RichText::new("▶ Run").color(design::INK))
                                .fill(GREEN),
                        )
                        .on_hover_text("Run playground.ts · Cmd+R")
                        .clicked()
                    {
                        self.start(draft, false);
                    }
                    if ui
                        .add_enabled(
                            !active && !checking && !context.formatting,
                            egui::Button::new("Debug"),
                        )
                        .on_hover_text("Pause at entry, then use breakpoints · F5")
                        .clicked()
                    {
                        self.start(draft, true);
                    }
                    if ui.add_enabled(active, egui::Button::new("Stop")).clicked() {
                        self.command("stop");
                    }
                    if self.debug && active {
                        ui.separator();
                        if ui
                            .add_enabled(paused, egui::Button::new("Continue"))
                            .on_hover_text("F5")
                            .clicked()
                        {
                            self.command("continue");
                        }
                        if ui
                            .add_enabled(
                                !paused && self.state == State::Running,
                                egui::Button::new("Pause"),
                            )
                            .clicked()
                        {
                            self.command("pause");
                        }
                        for (label, action, key) in [
                            ("Step over", "over", "F10"),
                            ("Step into", "into", "F11"),
                            ("Step out", "out", "Shift+F11"),
                        ] {
                            if ui
                                .add_enabled(paused, egui::Button::new(label))
                                .on_hover_text(key)
                                .clicked()
                            {
                                self.command(action);
                            }
                        }
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            !active && !checking && !context.formatting,
                            egui::Button::new(if checking {
                                "Checking…"
                            } else if self.guided {
                                "Check practice"
                            } else {
                                "Check lab"
                            }),
                        )
                        .on_hover_text("Validate the lesson contract · Cmd+Enter")
                        .clicked()
                    {
                        check = true;
                        self.panel = 2;
                    }
                    if ui.button("Task").clicked() {
                        if !compact {
                            self.task_docked = !self.task_docked;
                        } else {
                            self.task = true;
                        }
                    }
                    if ui
                        .button("Commands")
                        .on_hover_text("Command palette · Cmd+Shift+P")
                        .clicked()
                    {
                        self.palette.open();
                    }
                    ui.menu_button("View", |ui| {
                        if ui
                            .button(if self.sidebar {
                                "Hide explorer"
                            } else {
                                "Show explorer"
                            })
                            .clicked()
                        {
                            self.sidebar = !self.sidebar;
                            ui.close();
                        }
                        if ui.button("Find in file    Cmd+F").clicked() {
                            self.editor.open_find();
                            ui.close();
                        }
                        ui.separator();
                        ui.label(
                            RichText::new("Editor text size")
                                .size(11.0)
                                .color(design::MUTED),
                        );
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(self.font_size > 11.0, egui::Button::new("−"))
                                .clicked()
                            {
                                self.font_size -= 1.0;
                            }
                            ui.label(format!("{:.0} px", self.font_size));
                            if ui
                                .add_enabled(self.font_size < 22.0, egui::Button::new("+"))
                                .clicked()
                            {
                                self.font_size += 1.0;
                            }
                            if ui.small_button("Reset").clicked() {
                                self.font_size = 14.0;
                            }
                        });
                        ui.separator();
                        let maximum = (ctx.content_rect().height() * 0.52).clamp(210.0, 380.0);
                        ui.add(
                            egui::Slider::new(
                                &mut self.console_height,
                                if paused { 210.0 } else { 150.0 }..=maximum,
                            )
                            .integer()
                            .text("Output height"),
                        );
                    });
                    self.state_indicator(ui);
                });
            });
        if !self.task && !self.palette.is_open() && !checking && !context.formatting {
            let shortcut = egui::Modifiers {
                shift: true,
                command: true,
                ..Default::default()
            };
            if !active && ctx.input_mut(|input| input.consume_key(shortcut, egui::Key::I)) {
                format = Some(self.file.clone());
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::R)) && !active {
                self.start(draft, false);
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
                if paused {
                    self.command("continue");
                } else if !active {
                    self.start(draft, true);
                }
            }
            if paused {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F10)) {
                    self.command("over");
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::F11)) {
                    self.command("out");
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
                    self.command("into");
                }
            }
        }
        if compact && !active {
            egui::Panel::top("compact-exercise-task")
                .frame(
                    egui::Frame::new()
                        .fill(design::PANEL)
                        .inner_margin(egui::Margin::symmetric(12, 6)),
                )
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("Your task")
                        .id_salt((&lesson.id, "brief"))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&lesson.task)
                                    .size(12.5)
                                    .line_height(Some(19.0)),
                            );
                            ui.horizontal(|ui| {
                                if ui.small_button("Full task and hint").clicked() {
                                    self.task = true;
                                }
                                if ui.small_button("Worked solution").clicked() {
                                    solution = true;
                                }
                            });
                        });
                });
        }
        let max_console = (ctx.content_rect().height() * 0.52).clamp(145.0, 380.0);
        self.console_height = self
            .console_height
            .clamp(if paused { 210.0 } else { 150.0 }, max_console);
        self.output(ui, draft, checking, check_text, paused, max_console);
        if self.task_docked && !compact {
            egui::Panel::right("learning-task-panel")
                .default_size(280.0)
                .size_range(240.0..=380.0)
                .resizable(true)
                .frame(egui::Frame::new().fill(design::PANEL).inner_margin(16.0))
                .show(ui, |ui| {
                    let actions =
                        self.task_contents(ui, lesson, active || checking || context.formatting);
                    solution |= actions.0;
                    reset |= actions.1;
                });
        }
        if self.sidebar {
            egui::Panel::left("ide-sidebar").resizable(true).default_size(220.0).size_range(180.0..=330.0).frame(egui::Frame::new().fill(design::PANEL).inner_margin(12.0)).show(ui,|ui|{
                ui.horizontal(|ui|{ui.selectable_value(&mut self.debug_view,false,RichText::new("Explorer").size(11.5));ui.selectable_value(&mut self.debug_view,true,RichText::new("Run & Debug").size(11.5));});
                ui.separator();
                egui::ScrollArea::vertical().id_salt("ide-inspect-scroll").show(ui,|ui|{
                    if !self.debug_view {
                        design::eyebrow(ui,"WORKSPACE",design::MUTED);ui.add_space(6.0);
                        for file in ["answer.ts","playground.ts"]{if ui.selectable_label(self.file==file,RichText::new(file).monospace().size(12.0)).clicked(){self.file=file.into();}}
                        ui.add_space(20.0);ui.label(RichText::new(&lesson.title).size(12.0).color(design::MUTED));
                        ui.add_space(12.0);ui.label(RichText::new("answer.ts is your lab. playground.ts is for calls, experiments and logs.").size(12.0).color(design::MUTED));
                    } else {
                        design::eyebrow(ui,"CALL STACK",design::MUTED);ui.add_space(8.0);
                        if self.frames.is_empty(){ui.label(RichText::new("Start Debug to inspect a paused program.").size(12.0).color(design::MUTED));}
                        for (index,frame) in self.frames.iter().enumerate(){
                            if ui.selectable_label(self.frame==index,RichText::new(format!("{}\n{}:{}:{}",frame.name,frame.file,frame.line,frame.column)).monospace().size(11.0)).clicked(){self.frame=index;self.file=frame.file.clone();self.editor.reveal_line(frame.line);}
                        }
                        ui.add_space(18.0);design::eyebrow(ui,"VARIABLES",design::MUTED);ui.add_space(6.0);
                        if let Some(frame)=self.frames.get(self.frame){for variable in &frame.variables {
                            ui.horizontal_wrapped(|ui|{ui.label(RichText::new(&variable.name).monospace().size(11.5).color(design::ACCENT)).on_hover_text(&variable.scope);ui.add(egui::Label::new(RichText::new(&variable.value).monospace().size(11.5)).wrap().selectable(true));});
                        }}
                    }
                    ui.add_space(18.0);design::eyebrow(ui,"BREAKPOINTS",design::MUTED);ui.add_space(6.0);
                    let mut remove=None;
                    for (file,lines) in &draft.breakpoints {for line in lines {
                        ui.horizontal(|ui|{
                            if ui.selectable_label(false,RichText::new(format!("{file}:{line}")).monospace().size(11.0).color(RED)).clicked(){self.file=file.clone();self.editor.reveal_line(*line);}
                            if ui.add_enabled(!active,egui::Button::new("×").small()).on_hover_text("Remove breakpoint").clicked(){remove=Some((file.clone(),*line));}
                        });
                    }}
                    if let Some((file,line))=remove && let Some(points)=draft.breakpoints.get_mut(&file){points.remove(&line);changed=true;}
                    ui.add_space(4.0);ui.add_enabled_ui(!active,|ui|{
                        ui.horizontal(|ui|{
                            ui.add(egui::TextEdit::singleline(&mut self.breakpoint_line).hint_text("Line").desired_width(55.0));
                            if ui.small_button("Add").on_hover_text("Add a breakpoint in the active file").clicked()&&let Ok(line)=self.breakpoint_line.trim().parse::<usize>()&&line>0&&line<=4000{
                                draft.breakpoints.entry(self.file.clone()).or_default().insert(line);self.breakpoint_line.clear();changed=true;
                            }
                        });
                    });
                    ui.label(RichText::new("Click the gutter to toggle. Changes apply to the next Debug session.").size(11.0).color(design::MUTED));
                });
            });
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(12, 19, 29))
                    .inner_margin(0.0),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for file in ["answer.ts", "playground.ts"] {
                        let selected = self.file == file;
                        let response = ui.add(
                            egui::Button::new(
                                RichText::new(format!("TS  {file}"))
                                    .monospace()
                                    .size(12.0)
                                    .color(if selected {
                                        design::TEXT
                                    } else {
                                        design::MUTED
                                    }),
                            )
                            .fill(if selected {
                                design::PANEL
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(Stroke::NONE)
                            .corner_radius(0.0),
                        );
                        if selected {
                            ui.painter().line_segment(
                                [response.rect.left_bottom(), response.rect.right_bottom()],
                                Stroke::new(2.0, design::ACCENT),
                            );
                        }
                        if response.clicked() {
                            self.file = file.into();
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !active && !checking && !context.formatting,
                                egui::Button::new("Format"),
                            )
                            .on_hover_text("Prettier · Cmd+Shift+I")
                            .clicked()
                        {
                            format = Some(self.file.clone());
                        }
                        if active {
                            ui.label(
                                RichText::new("Read only · running snapshot")
                                    .size(11.0)
                                    .color(design::MUTED),
                            );
                        }
                    });
                });
                ui.separator();
                if context.formatting {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.small("Formatting with Prettier…");
                    });
                } else if let Some((notice, ok)) = context.format_notice {
                    if ok {
                        ui.label(RichText::new(notice).size(11.0).color(design::MUTED));
                    } else {
                        egui::CollapsingHeader::new(
                            RichText::new("Formatting failed · your code was preserved")
                                .size(11.5)
                                .color(AMBER),
                        )
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new(notice).monospace().size(11.0));
                                });
                        });
                    }
                }
                let source = if self.file == "answer.ts" {
                    draft.source.as_str()
                } else {
                    draft.playground.as_deref().unwrap_or(engine::PLAYGROUND)
                };
                if !self.palette.is_open() {
                    self.editor.find_bar(ui, source);
                }
                let height = (ui.available_height() - 42.0).max(100.0);
                let execution = if paused {
                    self.frames
                        .get(self.frame)
                        .filter(|f| f.file == self.file)
                        .map(|f| f.line)
                } else {
                    None
                };
                let source = if self.file == "answer.ts" {
                    &mut draft.source
                } else {
                    draft
                        .playground
                        .get_or_insert_with(|| engine::PLAYGROUND.into())
                };
                let points = draft.breakpoints.entry(self.file.clone()).or_default();
                changed |= self.editor.show(
                    ui,
                    source,
                    &format!("{}:{}", lesson.id, self.file),
                    editor::Options {
                        height: Some(height),
                        read_only: active,
                        workbench: true,
                        name: &self.file,
                        breakpoints: Some(points),
                        execution,
                        font_size: self.font_size,
                        ..Default::default()
                    },
                );
            });
        if self.task {
            let response = egui::Modal::new(egui::Id::new("ide-task"))
                .frame(
                    egui::Frame::new()
                        .fill(design::PANEL)
                        .stroke(Stroke::new(1.0, design::BORDER))
                        .corner_radius(10.0)
                        .inner_margin(24.0),
                )
                .show(&ctx, |ui| {
                    ui.set_width((ctx.content_rect().width() - 100.0).clamp(260.0, 620.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&lesson.title).font(design::display(20.0)));
                        if ui.button("Close").clicked() {
                            self.task = false;
                        }
                    });
                    ui.add_space(14.0);
                    let actions =
                        self.task_contents(ui, lesson, active || checking || context.formatting);
                    solution |= actions.0;
                    reset |= actions.1;
                });
            if response.should_close() || solution || reset {
                self.task = false;
            }
        }
        let commands = self.commands(checking);
        if let Some(action) = self.palette.show(&ctx, &commands) {
            match action {
                Action::Format => format = Some(self.file.clone()),
                Action::Solution => solution = true,
                Action::Reset => reset = true,
                _ => check |= self.apply(action, draft, checking),
            }
        }
        Response {
            changed,
            check,
            format,
            solution,
            reset,
        }
    }
}
