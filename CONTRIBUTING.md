# Contributing to TypeDojo

TypeDojo welcomes improvements to its explanations, guided exercises, compiler contracts, native editor, accessibility and documentation. Read [ROADMAP.md](ROADMAP.md) before proposing a new curriculum block or IDE capability.

## Local setup

Install Git, Python 3 and rustup (the Rust toolchain is pinned). Apple Silicon macOS 13.5+ is the packaged target. Bundling also requires Apple command-line tools. Node and TypeScript are fetched into the generated, app-owned toolchain.

```sh
python3 scripts/toolchain.py
export TYPEDOJO_HOME="$(mktemp -d -t typedojo-contrib)"
cargo run --locked
```

Use your temporary profile for development and screenshots. Never test resets, corrupt saves or storage migrations against your real progress. `TYPEDOJO_NODE` and `TYPEDOJO_TSC` accept executable paths when tools are for explicit developer overrides. The formatter is pinned and bundled; do not replace it with an arbitrary local Prettier version.

## Map of the code

- `content/course.json` / `src/course.rs`: independent lessons and compiler/runtime contracts.
- `content/learning.json` / `src/learning.rs`: structured readings, predictions and separate guided exercises.
- `src/compiler.rs`: answer checks; `src/engine.rs` / `runtime/bridge.cjs`: interactive Run/Debug.
- `src/formatter.rs` / `runtime/format.mjs`: offline Prettier and safe background results.
- `src/completion.rs`, `src/suggest.rs` and `runtime/complete.cjs`: semantic suggestions and exact-snapshot edits.
- `src/store.rs`: saved drafts, exact-source completion and locking.
- `src/reader.rs`, `src/editor.rs`, `src/ide.rs`, `src/main.rs`: native experience.

## Validation

```sh
python3 scripts/validate.py # complete required gates
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo run --locked -- --verify-content
cargo run --locked -- --verify-learning
node tools/format-content.mjs
```

Keep `TYPEDOJO_HOME` isolated for every command. The content verifiers require the actual Node/TypeScript tools: starters and negative answers must fail; reference and alternative solutions must pass. Guided verification also compiles explained examples and runs playgrounds. Record what ran and what did not; do not describe compilation alone as full runtime coverage.

For native UI changes, review the affected flow with fictional data, keyboard shortcuts, wide and compact windows. Do not generate automated presentation/snapshot tests. Changes to grading, storage or runtime boundaries need meaningful regression coverage.

## Write learning content

Follow [Writing TypeDojo content](docs/content-authoring.md) and the [learning contract](docs/learning-experience.md). Explain prerequisites and vocabulary before using them. Keep a reading, prediction, worked example, guided task and independent task aligned around one outcome. Add original explanations and cite technical references; do not copy handbook text into lessons.

Use `node tools/format-content.mjs --write` to format bundled code fields with the pinned formatter, then rerun the relevant verifiers. Do not rewrite stored learner drafts or alter vendored files and licenses just to satisfy whitespace preferences.

## Releases and runtime boundaries

`python3 scripts/bundle.py` builds the ad-hoc signed macOS ZIP and checksum, without installing over a learner’s app. Publication and catalog rollout are maintainer actions after review. Node and TypeScript remain separate prerequisites. The Inspector workbench is for trusted local practice code, not a hostile-code sandbox; Check must retain its separate restricted compiler/runtime path.

## Send a contribution

1. Open an issue for a substantial change so the learning goal or behavior can be agreed first. Small fixes can go straight to a PR.
2. Fork this repository, clone your fork, add `https://github.com/bujosa/typedojo.git` as `upstream`, and create a feature branch from `upstream/main`. Maintainer branches use `bujosa/<short-purpose>`.
3. Keep the change focused. Preserve existing learner data, unrelated changes and the app’s English UI. Use Conventional Commits, for example `fix: preserve saved drafts` or `docs: explain local setup`.
4. Open a PR against `bujosa/typedojo:main`. Explain the problem, resulting behavior, commands/results, manual review and any limits. Include a screenshot for a visual change, using fictional data and an isolated profile.

There is no hosted CI. Maintainers verify the relevant local gates before merge; a fork does not need access to a maintainer’s machine, token or Apple account. Never commit secrets, real learner drafts, local settings, build output or private screenshots. Use original or compatibly licensed assets and update `ATTRIBUTION.md` and notices when adding dependencies or artwork.

Be respectful in issues and reviews. Review may ask for a smaller scope, stronger examples, clearer explanations or additional checks. See `AGENTS.md` for the repository’s engineering agreements; it is also useful when working without an agent.

## Good first contributions

- Reproduce a reported issue with clear steps and a fictional profile.
- Improve a confusing explanation, hint, keyboard instruction or diagnostic.
- Fix a narrow accessibility or layout problem and include native visual evidence.
- Review an existing issue or roadmap item before starting a large feature.

Contributions are credited in the commit history and [GitHub contributors graph](https://github.com/bujosa/typedojo/graphs/contributors).
