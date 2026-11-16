//! Local process owner for the IDE. No completion authority: only compiler::check credits labs.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, Stdio},
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const PLAYGROUND: &str = "import * as answer from './answer';\n\n// Call your exported functions here and inspect the results.\nconsole.log('Your exports:', answer);\n";
pub type Breakpoints = BTreeMap<String, BTreeSet<usize>>;
#[derive(Clone, Debug, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub scope: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Frame {
    pub id: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub variables: Vec<Variable>,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "pausing")]
    Pausing,
    #[serde(rename = "paused")]
    Paused { reason: String, frames: Vec<Frame> },
    #[serde(rename = "log")]
    Log { level: String, text: String },
    #[serde(rename = "evaluation")]
    Evaluation { text: String },
    #[serde(rename = "commandError")]
    CommandError { text: String },
    #[serde(rename = "breakpoint")]
    Breakpoint {
        file: String,
        requested: usize,
        line: usize,
    },
    #[serde(rename = "finished")]
    Finished { success: bool },
    #[serde(rename = "failure")]
    Failure { text: String },
    #[serde(rename = "problems")]
    Problems { text: String },
    #[serde(rename = "stopped")]
    Stopped,
}
#[derive(Serialize)]
pub struct Request {
    pub debug: bool,
    pub breakpoints: Breakpoints,
    #[serde(skip)]
    pub answer: String,
    #[serde(skip)]
    pub playground: String,
}
pub struct Session {
    pub events: Receiver<Event>,
    commands: Sender<Value>,
}
impl Session {
    pub fn start(request: Request) -> Self {
        let (commands, input) = mpsc::channel();
        let (output, events) = mpsc::sync_channel(128);
        std::thread::spawn(move || {
            if let Err(text) = run(request, &input, &output) {
                let _ = output.send(Event::Failure { text });
            }
        });
        Self { events, commands }
    }
    pub fn command(&self, action: &str) {
        let _ = self.commands.send(json!({"action":action}));
    }
    pub fn evaluate(&self, frame: &str, expression: &str) {
        let _ = self
            .commands
            .send(json!({"action":"evaluate", "frame":frame, "expression":expression}));
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.command("stop");
    }
}
struct Work(PathBuf);
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn path_env() -> Result<std::ffi::OsString, String> {
    let mut paths: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    paths.extend(["/opt/homebrew/bin".into(), "/usr/local/bin".into()]);
    std::env::join_paths(paths).map_err(|e| e.to_string())
}
fn stopped(input: &Receiver<Value>) -> bool {
    match input.try_recv() {
        Ok(value) => value["action"] == "stop",
        Err(mpsc::TryRecvError::Disconnected) => true,
        _ => false,
    }
}
fn send(output: &SyncSender<Event>, event: Event) -> Result<(), String> {
    output.send(event).map_err(|_| "Workspace closed".into())
}
fn run(
    request: Request,
    input: &Receiver<Value>,
    output: &SyncSender<Event>,
) -> Result<(), String> {
    if request.answer.len() > 128 * 1024
        || request.playground.len() > 128 * 1024
        || request.answer.lines().count() > 4000
        || request.playground.lines().count() > 4000
    {
        return Err("Each workspace file is limited to 128 KiB and 4,000 lines.".into());
    }
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let work = Work(std::env::temp_dir().join(format!(
        "typedojo-ide-{}-{tick}-{sequence}",
        std::process::id()
    )));
    fs::create_dir(&work.0).map_err(|e| e.to_string())?;
    for (name, content) in [
        ("answer.ts", request.answer.as_str()),
        ("playground.ts", request.playground.as_str()),
        ("bridge.cjs", include_str!("../runtime/bridge.cjs")),
    ] {
        fs::write(work.0.join(name), content).map_err(|e| e.to_string())?;
    }
    fs::write(
        work.0.join("session.json"),
        serde_json::to_vec(&request).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(work.0.join("tsconfig.json"),r#"{"compilerOptions":{"strict":true,"noEmitOnError":true,"outDir":"out","sourceMap":true,"inlineSources":true,"exactOptionalPropertyTypes":true,"noUncheckedIndexedAccess":true,"target":"ES2022","module":"commonjs","types":[]},"files":["answer.ts","playground.ts"]}"#).map_err(|e|e.to_string())?;
    let log_path = work.0.join("compiler.log");
    let log = fs::File::create(&log_path).map_err(|e| e.to_string())?;
    let mut compiler = Process(crate::tooling::compiler()?.args(["-p","tsconfig.json","--pretty","false"])
        .current_dir(&work.0).env("PATH",path_env()?).stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e|e.to_string())?).stderr(log)
        .spawn().map_err(|e|format!("TypeScript compiler unavailable: {e}. Install Node.js and TypeScript or set TYPEDOJO_TSC."))?);
    let start = Instant::now();
    loop {
        if stopped(input) {
            let _ = compiler.0.kill();
            let _ = compiler.0.wait();
            return send(output, Event::Stopped);
        }
        if let Some(status) = compiler.0.try_wait().map_err(|e| e.to_string())? {
            if !status.success() {
                let mut bytes = Vec::new();
                fs::File::open(&log_path)
                    .map_err(|e| e.to_string())?
                    .take(64 * 1024)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                send(
                    output,
                    Event::Problems {
                        text: String::from_utf8_lossy(&bytes).into_owned(),
                    },
                )?;
                return send(output, Event::Finished { success: false });
            }
            break;
        }
        if start.elapsed() > Duration::from_secs(20) {
            return Err("Compilation exceeded 20 seconds. Simplify the types and retry.".into());
        }
        if fs::metadata(&log_path).is_ok_and(|m| m.len() > 1_048_576) {
            return Err("Compiler output exceeded 1 MiB.".into());
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    let stderr = fs::File::create(work.0.join("node.log")).map_err(|e| e.to_string())?;
    // Node's permission mode prohibits inspector sessions. The VM exposes no host API, but is
    // educational isolation only. Existing lab checks retain their separate permission-mode runner.
    let mut child = Process(
        crate::tooling::node()?
            .args([
                "--max-old-space-size=96",
                "--disable-proto=throw",
                "bridge.cjs",
            ])
            .current_dir(&work.0)
            .env("PATH", path_env()?)
            .env_remove("NODE_OPTIONS")
            .env_remove("NODE_PATH")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .map_err(|e| {
                format!("Node.js unavailable: {e}. Install Node.js 22.13+ or set TYPEDOJO_NODE.")
            })?,
    );
    let stdout = child.0.stdout.take().ok_or("Runtime output unavailable")?;
    let mut stdin = child.0.stdin.take().ok_or("Runtime input unavailable")?;
    let (lines_tx, lines_rx) = mpsc::sync_channel(64);
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut bytes = Vec::new();
            let read = (&mut reader).take(64 * 1024).read_until(b'\n', &mut bytes);
            match read {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if bytes.last() != Some(&b'\n') {
                        let _ = lines_tx.send(Err("Runtime message exceeded 64 KiB".to_string()));
                        break;
                    }
                    if lines_tx
                        .send(
                            serde_json::from_slice::<Event>(&bytes)
                                .map_err(|e| format!("Runtime protocol error: {e}")),
                        )
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });
    let started = Instant::now();
    let mut active = Duration::ZERO;
    let mut last = Instant::now();
    let mut paused = false;
    loop {
        let now = Instant::now();
        if !paused {
            active += now.duration_since(last);
        }
        last = now;
        while let Ok(command) = input.try_recv() {
            if command["action"] == "stop" {
                let _ = child.0.kill();
                let _ = child.0.wait();
                return send(output, Event::Stopped);
            }
            writeln!(stdin, "{command}").map_err(|e| e.to_string())?;
            stdin.flush().map_err(|e| e.to_string())?;
        }
        while let Ok(event) = lines_rx.try_recv() {
            let event = event?;
            match &event {
                Event::Paused { .. } | Event::Pausing => paused = true,
                Event::Running => paused = false,
                Event::Finished { .. } | Event::Failure { .. } => {
                    // Reap before declaring completion so no process or temp files outlive a result.
                    let _ = child.0.kill();
                    let _ = child.0.wait();
                    return send(output, event);
                }
                _ => {}
            }
            send(output, event)?;
        }
        if active > Duration::from_secs(5) {
            return Err("Execution exceeded 5 seconds of running time. Check for an infinite loop, or use Stop.".into());
        }
        if started.elapsed() > Duration::from_secs(600) {
            return Err(
                "Debug session reached its 10-minute limit. Start another session to continue."
                    .into(),
            );
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            // The reader may still be draining stdout after process exit.
            match lines_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    let terminal = matches!(event, Event::Finished { .. } | Event::Failure { .. });
                    send(output, event)?;
                    if terminal {
                        return Ok(());
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    let mut bytes = Vec::new();
                    fs::File::open(work.0.join("node.log"))
                        .map_err(|e| e.to_string())?
                        .take(4096)
                        .read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())?;
                    return Err(format!(
                        "Runtime exited unexpectedly ({status}). {}",
                        String::from_utf8_lossy(&bytes)
                    ));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(15));
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    fn request(answer: &str, playground: &str, debug: bool) -> Request {
        Request {
            answer: answer.into(),
            playground: playground.into(),
            debug,
            breakpoints: Breakpoints::new(),
        }
    }
    fn next(session: &Session) -> Event {
        session
            .events
            .recv_timeout(Duration::from_secs(25))
            .unwrap()
    }
    #[test]
    fn run_streams_logs_without_crediting_labs_and_reports_compile_and_runtime_errors() {
        let session = Session::start(request(
            "export const value = 42;",
            "import {value} from './answer'; console.log('answer', value); console.warn('careful');",
            false,
        ));
        let mut logs = Vec::new();
        loop {
            match next(&session) {
                Event::Log { level, text } => logs.push((level, text)),
                Event::Finished { success } => {
                    assert!(success);
                    break;
                }
                Event::Failure { text } => panic!("{text}"),
                _ => {}
            }
        }
        assert!(logs.iter().any(|(l, t)| l == "log" && t.contains("42")));
        assert!(
            logs.iter()
                .any(|(l, t)| l == "warning" && t.contains("careful"))
        );
        let session = Session::start(request(
            "export const value: number = 'wrong';",
            PLAYGROUND,
            false,
        ));
        assert!(
            matches!(next(&session),Event::Problems{text} if text.contains("answer.ts(1,")&&text.contains("TS2322"))
        );
        assert!(matches!(next(&session), Event::Finished { success: false }));
        let session = Session::start(request(
            "export {};",
            "throw new Error('exercise failure');",
            false,
        ));
        let mut error = false;
        loop {
            match next(&session) {
                Event::Log { level, text } => {
                    error |= level == "error" && text.contains("exercise failure")
                }
                Event::Finished { success } => {
                    assert!(!success);
                    break;
                }
                Event::Failure { text } => panic!("{text}"),
                _ => {}
            }
        }
        assert!(error);
    }
    #[test]
    fn debugger_maps_breakpoints_inspects_frames_evaluates_and_steps() {
        let mut req = request(
            "export function double(n:number):number {\n const value = n*2;\n console.log('doubling', n);\n return value;\n}\n",
            "import {double} from './answer';\nconst result = double(21);\nconsole.log('Result', result);\n",
            true,
        );
        req.breakpoints.insert("answer.ts".into(), [3].into());
        let session = Session::start(req);
        let mut breakpoint = false;
        let mut inspected = false;
        let mut evaluated = false;
        let mut stepped = false;
        loop {
            match next(&session) {
                Event::Breakpoint {
                    file,
                    requested,
                    line,
                } => {
                    assert_eq!((file.as_str(), requested, line), ("answer.ts", 3, 3));
                    breakpoint = true;
                }
                Event::Paused { frames, .. } => {
                    let frame = &frames[0];
                    if frame.file == "answer.ts" && frame.line == 3 {
                        assert!(frames.len() >= 2);
                        assert!(
                            frame
                                .variables
                                .iter()
                                .any(|v| v.name == "n" && v.value == "21")
                        );
                        assert!(
                            frame
                                .variables
                                .iter()
                                .any(|v| v.name == "value" && v.value == "42")
                        );
                        inspected = true;
                        session.evaluate(&frame.id, "value + n");
                    } else if inspected {
                        stepped = true;
                        session.command("continue");
                    } else {
                        session.command("continue");
                    }
                }
                Event::Evaluation { text } => {
                    assert_eq!(text, "63");
                    evaluated = true;
                    session.command("over");
                }
                Event::Finished { success } => {
                    assert!(success);
                    break;
                }
                Event::Failure { text } | Event::CommandError { text } => panic!("{text}"),
                _ => {}
            }
        }
        assert!(breakpoint && inspected && evaluated && stepped);
    }
    #[test]
    fn cancellation_and_infinite_loops_end_sessions() {
        let session = Session::start(request("export {};", "while(true) {}", false));
        loop {
            match next(&session) {
                Event::Finished { success: false } | Event::Failure { .. } => break,
                Event::Finished { success: true } => panic!("loop succeeded"),
                _ => {}
            }
        }
        let session = Session::start(request("export const n=1;", PLAYGROUND, true));
        loop {
            if matches!(next(&session), Event::Paused { .. }) {
                break;
            }
        }
        session.command("stop");
        assert!(matches!(next(&session), Event::Stopped));
    }
    #[test]
    fn stepping_into_and_out_uses_original_typescript_frames() {
        let session = Session::start(request(
            "export function double(n:number) {\n const value = n*2;\n return value;\n}\n",
            "import {double} from './answer';\nconst result = double(21);\nconsole.log(result);\n",
            true,
        ));
        let mut entered = false;
        let mut returned = false;
        loop {
            match next(&session) {
                Event::Paused { frames, .. } => {
                    let frame = &frames[0];
                    if frame.file == "answer.ts" {
                        entered = true;
                        session.command("out");
                    } else if entered {
                        returned = true;
                        session.command("continue");
                    } else if frame.line == 1 {
                        session.command("over");
                    } else {
                        session.command("into");
                    }
                }
                Event::Finished { success } => {
                    assert!(success);
                    break;
                }
                Event::Failure { text } | Event::CommandError { text } => panic!("{text}"),
                _ => {}
            }
        }
        assert!(entered && returned);
    }
    #[test]
    fn pause_interrupts_running_code_and_stop_reaps_it() {
        let session = Session::start(request(
            "export function spin(){\n let n=0;\n while(true){n++;}\n}\n",
            "import {spin} from './answer';\nspin();\n",
            true,
        ));
        loop {
            if matches!(next(&session), Event::Paused { .. }) {
                break;
            }
        }
        session.command("continue");
        loop {
            if matches!(next(&session), Event::Running) {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
        session.command("pause");
        loop {
            match next(&session) {
                Event::Paused { frames, .. } => {
                    assert_eq!(frames[0].file, "answer.ts");
                    break;
                }
                Event::Failure { text } | Event::CommandError { text } => panic!("{text}"),
                _ => {}
            }
        }
        session.command("stop");
        assert!(matches!(next(&session), Event::Stopped));
    }
    #[test]
    fn duplicate_entry_breakpoint_and_runtime_limits_are_explicit() {
        let mut req = request(
            "export const n=1;",
            "import {n} from './answer';\nconsole.log(n);\n",
            true,
        );
        req.breakpoints.insert("playground.ts".into(), [1].into());
        let session = Session::start(req);
        loop {
            match next(&session) {
                Event::Paused { .. } => session.command("continue"),
                Event::Finished { success } => {
                    assert!(success);
                    break;
                }
                Event::Failure { text } => panic!("{text}"),
                _ => {}
            }
        }
        let session = Session::start(request("export {};", "console.log(typeof process);", false));
        assert!(
            matches!(next(&session),Event::Problems{text} if text.contains("Cannot find name 'process'"))
        );
        assert!(matches!(next(&session), Event::Finished { success: false }));
        let session = Session::start(request(
            "export {};",
            "console.log('before'); for(let i=0;i<510;i++)console.log(i);",
            false,
        ));
        let mut logs = 0;
        let mut capped = false;
        loop {
            match next(&session) {
                Event::Log { text, .. } => {
                    logs += 1;
                    capped |= text.contains("limit reached");
                }
                Event::Finished { success } => {
                    assert!(success);
                    break;
                }
                Event::Failure { text } => panic!("{text}"),
                _ => {}
            }
        }
        assert!(capped && logs == 501);
    }
}
