# TypeDojo
Native Rust/egui TypeScript practice app in the Keiko family. English UI and content.

- `src/art.rs`: stable lesson-keyed illustrations, background decode and bounded texture cache. Run --verify-art for asset delivery checks.
- `src/course.rs`: pure curriculum model, embedded `content/course.json`.
- `src/learning.rs` / `content/learning.json`: 53 structured guides and separate guided exercise contracts.
- `src/reader.rs`: glossary, worked examples and explained predictions.
- `src/formatter.rs` / `runtime/format.mjs`: pinned offline Prettier; background, snapshot-safe, no learner config.
- `src/store.rs`: local drafts and completion; `TYPEDOJO_HOME` isolates all learner data.
- `src/compiler.rs`: real `tsc`, strict; background jobs only. Execute function samples only through the restricted, time-limited Node VM.
- `src/engine.rs` / `runtime/bridge.cjs`: interactive Run/Debug sessions; real Inspector worker, source maps, no TCP port. Educational VM for trusted local code; Inspector prevents Node permission mode. Never route Check lab through this engine.
- `src/ide.rs`: IDE layout/session presentation; Run never credits completion.
- `src/main.rs`: thin egui UI; command work stays off its thread.
- No panics, unwrap, unsafe, webview, or async runtime in shipped code.
- Never overwrite unreadable progress. Atomic writes. No writes to another dojo's data.
- Content checks must reject starters, accept solutions and an alternative, reject negative cases.
- Before commits: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `cargo run --locked -- --verify-content`, `cargo run --locked -- --verify-learning`,
  `node tools/format-content.mjs`. Verification requires local Node and tsc.
- Use `TYPEDOJO_HOME` in tests and visual review, never the learner's real data.
- No public release or catalog installation entry until a verified signed bundle exists.
- Conventional Commits, bujosa/ branches. No hosted CI.
