# Validation — initial local build

2026-11-14, macOS Apple silicon.

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: four domain/storage tests passed. Includes corrupt/newer-file preservation,
  exclusive session lock, persistence round-trip and invalidation after an edit.
- `cargo run -- --verify-content`: all 53 lessons passed six cases each (318 real-compiler
  checks). Twenty functional/project labs additionally run their examples when compilation passes.
- `python3 scripts/bundle.py`: release binary built with home paths remapped, leak check passed,
  macOS app and original icon generated, ad-hoc code signature verified, ZIP and SHA-256 generated.
- Native visual review: wide 1440x900 and compact 820x700. Compact view separates Learn/Practice.
- Native interaction review in an isolated TYPEDOJO_HOME: starter rejected, compiler diagnostics
  visible, editor input, Cmd+Enter, pass count, edit invalidates pass, hints, lesson search,
  tracker project type/runtime pass. Quit/reopen restored selected project, exact source and pass.
- No automated presentation/UI tests were added.

Known limits: initial single-file curriculum, no autocomplete, async runtime grader, multiple
profiles or spaced review yet. Professional multi-file projects and further subject coverage are
explicitly outstanding in ROADMAP.md. The bundle is not notarized. GitHub publication and the live
Keiko catalog entry have not been performed.

Keiko and PodDojo were reviewed without source edits. Both `cargo xtask ci` gates passed. Keiko's
`cargo test -p keiko-native -- --ignored` passed both real install/update tests (PodDojo and Keiko)
in isolated temporary folders. PodDojo's 40 lessons, 190 quiz questions, 159 lab steps and four exams
passed content validation. Its real cluster labs were not all re-executed in this session.
