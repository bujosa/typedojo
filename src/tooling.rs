//! Resolve only the app-owned toolchain, with explicit developer overrides.
use std::{
    path::{Path, PathBuf},
    process::Command,
};
pub fn resource(relative: &str) -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    if let Some(root) = executable.parent().and_then(Path::parent) {
        let p = root.join("Resources").join(relative);
        if p.exists() {
            return Ok(p);
        }
    }
    #[cfg(debug_assertions)]
    {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/toolchain")
            .join(relative);
        if p.exists() {
            return Ok(p);
        }
    }
    Err("Bundled tools are missing. Reinstall TypeDojo. Developers: run python3 scripts/toolchain.py.".into())
}
pub fn node() -> Result<Command, String> {
    let path = std::env::var_os("TYPEDOJO_NODE")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| resource("node/bin/node"))?;
    let mut c = Command::new(path);
    c.env_remove("NODE_OPTIONS").env_remove("NODE_PATH");
    Ok(c)
}
pub fn compiler() -> Result<Command, String> {
    if let Some(path) = std::env::var_os("TYPEDOJO_TSC") {
        return Ok(Command::new(path));
    }
    let mut c = node()?;
    c.arg(resource("typescript/lib/tsc.js")?);
    Ok(c)
}
