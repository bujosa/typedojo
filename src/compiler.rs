//! Compile contracts, then run functional labs in a restricted, time-limited Node VM.
//! No shell commands, package downloads, or learner imports in runtime labs.
use crate::course::Lesson;
use std::{
    fs,
    io::Read,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

pub struct Report {
    pub passed: bool,
    pub text: String,
}
struct Work(PathBuf);
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn check(lesson: &Lesson, source: &str) -> Result<Report, String> {
    // Do not let directives in a draft turn off checking. Contract files have their own expected
    // error assertions. This deliberately conservative teaching rule is not a security sandbox.
    if ["@ts-", "<reference"]
        .iter()
        .any(|marker| source.contains(marker))
    {
        return Ok(Report { passed: false, text: "Remove TypeScript suppression/reference directives from your answer. Solve the contract without disabling the compiler.".into() });
    }
    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let work =
        Work(std::env::temp_dir().join(format!("typedojo-check-{}-{tick}", std::process::id())));
    fs::create_dir(&work.0).map_err(|e| e.to_string())?;
    fs::write(work.0.join("answer.ts"), source).map_err(|e| e.to_string())?;
    let prelude = "import * as s from './answer';\ntype Equal<A,B> = (<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2) ? true : false;\ntype Assert<T extends true> = T;\n";
    fs::write(
        work.0.join("contract.ts"),
        format!("{prelude}{}", lesson.checks),
    )
    .map_err(|e| e.to_string())?;
    fs::write(work.0.join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"noEmit":false,"outDir":"out","exactOptionalPropertyTypes":true,"noUncheckedIndexedAccess":true,"target":"ES2022","module":"commonjs","types":[],"skipLibCheck":false},"files":["answer.ts","contract.ts"]}"#).map_err(|e| e.to_string())?;
    let log_path = work.0.join("compiler.log");
    let log = fs::File::create(&log_path).map_err(|e| e.to_string())?;
    let mut paths: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    let mut child = crate::tooling::compiler()?.args(["--project", "tsconfig.json", "--pretty", "false"])
        .current_dir(&work.0).env("PATH", std::env::join_paths(paths).map_err(|e| e.to_string())?)
        .stdin(Stdio::null()).stdout(log.try_clone().map_err(|e| e.to_string())?).stderr(log)
        .spawn().map_err(|e| format!("TypeScript compiler unavailable: {e}. Install Node.js and TypeScript (npm install -g typescript), then retry. TYPEDOJO_TSC can select an absolute compiler path."))?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Compiler exceeded 20 seconds. Simplify your types and retry.".into());
        }
        if fs::metadata(&log_path)
            .map(|m| m.len() > 1_048_576)
            .unwrap_or(false)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Compiler output exceeded 1 MiB. Simplify the draft and retry.".into());
        }
        std::thread::sleep(Duration::from_millis(40));
    };
    let mut bytes = Vec::new();
    fs::File::open(log_path)
        .map_err(|e| e.to_string())?
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let output = String::from_utf8_lossy(&bytes);
    if status.success() && !lesson.runtime.is_empty() {
        return runtime(&work, lesson);
    }
    Ok(Report {
        passed: status.success(),
        text: if status.success() {
            "Type contract passed. Valid examples compile and invalid examples are rejected. This check does not execute or verify runtime behavior.".into()
        } else {
            format!(
                "Not yet — read the diagnostics below.\nanswer.ts is your code; contract.ts tests the requested public API.\n\n{output}"
            )
        },
    })
}

/// Real compiler integration: every starter fails and both reference answers pass. Empty and
/// unrelated answers fail too, proving that mere successful compilation cannot earn completion.
pub fn verify(lessons: &[Lesson]) -> Result<(), String> {
    for lesson in lessons {
        for (name, source, expected) in [
            ("starter", lesson.starter.as_str(), false),
            ("solution", &lesson.solution, true),
            ("alternative", &lesson.alternative, true),
            ("missing exports", "export {};", false),
            ("wrong shape", "export const unrelated = 1;", false),
            ("negative solution", &lesson.negative, false),
        ] {
            let report = check(lesson, source)?;
            if report.passed != expected {
                return Err(format!(
                    "{} / {name}: unexpected result\n{}",
                    lesson.id, report.text
                ));
            }
        }
        println!(
            "{}: starter rejected, solutions accepted, missing/wrong exports rejected",
            lesson.id
        );
    }
    Ok(())
}

fn runtime(work: &Work, lesson: &Lesson) -> Result<Report, String> {
    // The context contains no process, require, timers, host callbacks, filesystem or network API.
    // It is educational isolation, not a security boundary for adversarial code.
    const RUNNER: &str = r#"const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync('out/answer.js', 'utf8');
const checks = fs.readFileSync('runtime-checks.js', 'utf8');
try {
 vm.runInNewContext('const exports = {};\n' + source + '\n' + checks,
  Object.create(null), {timeout:1500, contextCodeGeneration:{strings:false,wasm:false}, microtaskMode:'afterEvaluate'});
 console.log('Runtime examples passed.');
} catch(error) { console.error(String(error)); process.exitCode=1; }
"#;
    fs::write(work.0.join("runner.cjs"), RUNNER).map_err(|e| e.to_string())?;
    fs::write(work.0.join("runtime-checks.js"), &lesson.runtime).map_err(|e| e.to_string())?;
    let log_path = work.0.join("runtime.log");
    let log = fs::File::create(&log_path).map_err(|e| e.to_string())?;
    let mut child = crate::tooling::node()?
        .args([
            "--permission",
            "--allow-fs-read=.",
            "--max-old-space-size=96",
            "runner.cjs",
        ])
        .current_dir(&work.0)
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log)
        .spawn()
        .map_err(|e| format!("Bundled runtime could not start: {e}"))?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
        if start.elapsed() > Duration::from_secs(4) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(Report {
                passed: false,
                text: "Runtime check timed out. Look for an infinite loop or recursive call."
                    .into(),
            });
        }
        std::thread::sleep(Duration::from_millis(30));
    };
    let mut bytes = Vec::new();
    fs::File::open(log_path)
        .map_err(|e| e.to_string())?
        .take(16 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let output = String::from_utf8_lossy(&bytes);
    Ok(Report {
        passed: status.success(),
        text: if status.success() {
            "Type contracts and runtime examples passed. Try more edge cases as you practice."
                .into()
        } else {
            format!("Runtime examples did not pass:\n{output}")
        },
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn suppression_cannot_earn_completion() {
        let lesson = crate::course::load().unwrap().remove(0);
        assert!(
            !super::check(&lesson, "// @ts-nocheck\nexport const x=1;")
                .unwrap()
                .passed
        );
    }
}
