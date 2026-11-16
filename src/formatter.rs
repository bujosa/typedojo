//! Real pinned Prettier, bundled offline. Formats data; it never executes the learner's source.
use std::{
    fs,
    io::Read,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub const VERSION: &str = "3.6.2";
struct Work(PathBuf);
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[derive(serde::Deserialize)]
pub struct Formatted {
    pub source: String,
    pub breakpoints: std::collections::BTreeSet<usize>,
}
pub fn format(
    source: &str,
    breakpoints: &std::collections::BTreeSet<usize>,
) -> Result<Formatted, String> {
    if source.len() > 128 * 1024 {
        return Err("Format supports files up to 128 KiB.".into());
    }
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let work = Work(std::env::temp_dir().join(format!(
        "typedojo-format-{}-{tick}-{seq}",
        std::process::id()
    )));
    fs::create_dir(&work.0).map_err(|e| e.to_string())?;
    for (name, bytes) in [
        (
            "standalone.mjs",
            &include_bytes!("../assets/vendor/prettier/standalone.mjs")[..],
        ),
        (
            "typescript.mjs",
            &include_bytes!("../assets/vendor/prettier/typescript.mjs")[..],
        ),
        (
            "estree.mjs",
            &include_bytes!("../assets/vendor/prettier/estree.mjs")[..],
        ),
        (
            "format.mjs",
            include_str!("../runtime/format.mjs").as_bytes(),
        ),
        ("input.ts", source.as_bytes()),
    ] {
        fs::write(work.0.join(name), bytes).map_err(|e| e.to_string())?;
    }
    fs::write(
        work.0.join("positions.json"),
        serde_json::to_vec(breakpoints).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let log = fs::File::create(work.0.join("error.log")).map_err(|e| e.to_string())?;
    let mut paths: Vec<_> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    let mut child = crate::tooling::node()?
        .args([
            "--permission",
            "--allow-fs-read=.",
            "--allow-fs-write=.",
            "--max-old-space-size=96",
            "format.mjs",
        ])
        .current_dir(&work.0)
        .env(
            "PATH",
            std::env::join_paths(paths).map_err(|e| e.to_string())?,
        )
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .map_err(|e| format!("Prettier needs Node.js 22.13+: {e}. Your code was preserved."))?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
        if start.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Formatting timed out. Your code was preserved.".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    if !status.success() {
        let mut bytes = Vec::new();
        fs::File::open(work.0.join("error.log"))
            .map_err(|e| e.to_string())?
            .take(4096)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        return Err(format!(
            "Prettier could not format this code. Fix its syntax and retry.\n{}",
            String::from_utf8_lossy(&bytes)
        ));
    }
    let path = work.0.join("output.json");
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 256 * 1024 {
        return Err("Formatted output exceeded 256 KiB. Your code was preserved.".into());
    }
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    #[test]
    fn real_prettier_formats_typescript_and_is_idempotent() {
        let raw = "export function greet(name:string){return {text:'Hello '+name,ok:true}}";
        let formatted = super::format(raw, &Default::default()).unwrap().source;
        assert!(formatted.contains("greet(name: string)"));
        assert!(formatted.contains("  return { text: \"Hello \" + name, ok: true };"));
        assert_eq!(
            super::format(&formatted, &Default::default())
                .unwrap()
                .source,
            formatted
        );
        assert!(super::format("export const = ;", &Default::default()).is_err());
        assert!(
            super::format("", &Default::default())
                .unwrap()
                .source
                .is_empty()
        );
        let result = super::format(
            "export function f(){return 1;}\nconsole.log(f());",
            &[2].into(),
        )
        .unwrap();
        let line = result.breakpoints.iter().next().copied().unwrap();
        assert!(
            result
                .source
                .lines()
                .nth(line - 1)
                .unwrap()
                .contains("console.log")
        );
    }
}
