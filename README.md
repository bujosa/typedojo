# TypeDojo

A native TypeScript practice app in the Keiko family. Learn the vocabulary, trace a worked example, solve a guided exercise, then apply it in an
independent lab. Check with the real TypeScript compiler and resume both saved drafts next time.

**0.1 is an initial course, not a complete professional qualification.** Fifty-three lessons introduce
JavaScript basics and progress to advanced type transformations and professional API patterns.
Every lesson has a structured guide, a separately graded guided exercise, an independent lab,
hints, worked solutions and real compiler contracts.
Twenty function and project labs also check runtime examples. Further projects and coverage are tracked in
[ROADMAP.md](ROADMAP.md).

## Install

Download **TypeDojo-v0.2.0-macos-arm64.zip** from the [latest release](https://github.com/bujosa/typedojo/releases/latest), extract it and move TypeDojo.app to `~/Applications`. Keiko also installs, verifies and updates TypeDojo from its catalog.

The initial binary supports macOS Apple Silicon. It is ad-hoc signed and not notarized; macOS may ask you to approve its first launch in System Settings → Privacy & Security. Node.js 24.22.3 and TypeScript 5.9.3 are bundled with the app. The 0.2 runtime requires macOS 13.5 or newer. No separate Node/npm installation is needed.

## Start from source

Apple Silicon macOS 13.5+, Python 3 (build tooling) and Rust (pinned by `rust-toolchain.toml`):

```sh
python3 scripts/toolchain.py
cargo run
```

You do not need a database, Docker, a server or an account. The signed app includes its verified toolchain and works offline. Developer setup downloads checksum-pinned archives into `target/toolchain`; it never modifies system Node/npm.

- Choose a lesson in the left rail; search by title or section.
- **Learn** explains the outcome, prerequisite concepts and vocabulary, then traces a colored TypeScript example.
- Predict its result and read the explanation, then solve **Guided practice** with a smaller task and a specific hint.
- Transfer the concept to **Independent lab**. Each stage keeps its own draft and exact-source pass evidence.
- Guided practice never increases the lab counter. You can revisit any stage without losing your independent answer.
- **Check lab** / **Cmd+Enter** runs strict compiler checks and, for selected function labs, runtime examples.
- Compiler diagnostics name `answer.ts` (your code) or `contract.ts` (the required public contract).
- A pass applies to the exact checked answer. Editing it makes the lab pending again.
- Hints and worked solutions do not automatically complete a lab.
- Drafts and the selected lesson are saved locally; save errors are shown and can be retried.

## IDE focus mode

Choose **Focus on code** for a native workspace with two saved files:
`answer.ts` contains the lesson implementation; `playground.ts` imports it and calls your functions.
For example, after exporting `greet`, use `import { greet } from './answer'; console.log(greet('Ada'));`.

- **Run / Cmd+R** compiles both files with strict TypeScript and streams real console output.
- **Debug / F5** pauses at entry. Click the gutter or use the sidebar to set breakpoints before starting.
- **Continue / F5**, **Step over / F10**, **Step into / F11**, **Step out / Shift+F11**, **Pause** and **Stop** control V8.
- Inspect variables and stack frames; evaluate side-effect-free expressions in the selected frame.
- **Suggest / Ctrl+Space** opens semantic suggestions for the current code and the other workspace file. Typing a member access or identifier also requests them automatically. Use Up/Down, then Tab or Enter; Escape dismisses. Acceptance is undoable.
- **Format / Cmd+Shift+I** formats the active file with bundled Prettier 3.6.2, offline. Syntax errors preserve your code; Undo restores the previous text. Edits during formatting discard its stale result.
- **Commands / Cmd+Shift+P** opens a searchable keyboard command palette with state-aware actions.
- **Find / Cmd+F** highlights literal case-sensitive matches in the active file; Enter / Shift+Enter moves through them.
- **View** adjusts editor text size; console filters narrow logs by text or severity, and Copy copies visible output.
- **Problems** links compiler diagnostics to their file and line. **Task** shows the current challenge beside the editor in wide windows, or in a dialog in compact windows.
- Use **Expand / Restore** to resize the Console. Toggle the sidebar for more editor space.
- Editing is locked to the running snapshot. Stop before changing code or breakpoints.
- Leaving Focus, switching lessons or closing the app stops the active session.
- **Run** success does not credit a lesson. **Check lab / Cmd+Enter** validates the contract.

The workbench supports these two local TypeScript files, synchronous code and Promise microtasks.
There are no npm packages, filesystem/network APIs or timers in the VM. Its Inspector worker opens
no debugging TCP port. Node permission mode cannot be used with Inspector sessions; the interactive
workbench is for trusted local practice code and is not a hostile-code sandbox. Existing runtime lab
checks retain their own permission-mode process. Active execution is limited to five seconds,
debug sessions to ten minutes, and console output to 500 messages.

See [docs/ide-engine.md](docs/ide-engine.md) for the engine contract and validation.

The [native E2E report](docs/e2e-validation-2026-11-15.md) records the complete 53-lesson UI traversal,
the formatter undo fix, integration checks and the remaining distribution/cluster coverage limits.

Every lesson includes its own original concept illustration, embedded locally. The reading layout gives code and explanations more space, with an explicit active stage and direct practice action. Artwork prompts and file hashes are recorded in `assets/art/prompts-v1.json`.

## Learning path

| Block | Labs | Topics |
|---|---:|---|
| First steps | 16 | Values, variables, arrays, objects, functions, branching, iteration |
| TypeScript foundations | 6 | Optional fields, literals, tuples, readonly, unknown, discriminated unions |
| Reusable types | 16 | Generic types/functions, constraints, keyof, indexed access, mapped and utility types |
| Advanced practice | 6 | Conditional types, infer, template literals, key remapping, async, exhaustiveness |
| Professional patterns | 6 | Request states, branded IDs, validation, repositories, event maps, API capstone |
| Applied projects | 3 | Study tracker, boundary parser, searchable paginated catalog |

Checks are learning feedback. Type contracts cannot prove algorithm correctness; runtime examples
cover specific cases. A pass does not mean production-ready code or mastery of all TypeScript.


## Local data and isolation

Default: `~/Library/Application Support/TypeDojo/progress.json`. `TYPEDOJO_HOME` selects another folder
for testing. Only one instance may write to a folder. After a crash, close any remaining instance
and remove its `session.lock` to reopen. Corrupt or newer progress is preserved and saving is disabled
instead of silently replacing it.

**Check lab:** compiler work runs in disposable owned folders, has a 20-second deadline, and never compiles your
existing projects. Runtime labs use a separate Node process (96 MiB heap, four-second deadline),
a VM without host APIs, disabled string/Wasm code generation, and Node permission flags allowing
reads only in that temporary folder. This limits mistakes; **Node VM and permission flags are not
a security sandbox for malicious code**. Use only your own trusted practice answers.

`TYPEDOJO_TSC` and `TYPEDOJO_NODE` are optional developer overrides. Normal signed-app execution uses its bundled tools, including for completion. Homebrew is not required.

## Development

```sh
python3 scripts/toolchain.py
python3 scripts/validate.py # all required local gates
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --locked -- --verify-art
cargo run --locked -- --verify-content
cargo run --locked -- --verify-learning
node tools/format-content.mjs # check all bundled TypeScript code fields
python3 scripts/bundle.py  # build, sign ad-hoc, zip, SHA256SUMS.txt
```

Content verification uses the actual compiler for every starter, reference solution, alternative,
missing exports, wrong exports and negative answer. Functional negatives include implementations
whose signatures are correct but whose behavior is wrong. Learning verification checks six cases
for each of the 53 guided exercises, compiles every worked example, and runs every guided playground.
Use `node tools/format-content.mjs --write` to format repository content with the same pinned Prettier.
Stored learner drafts are never automatically rewritten during an update.

See [docs/learning-experience.md](docs/learning-experience.md) for the learning and formatting contracts.

`TYPEDOJO_SCREENSHOT=/tmp/typedojo.png` captures a native frame and closes the app;
`TYPEDOJO_WINDOW=820x700` checks the compact layout. Always set `TYPEDOJO_HOME` to a temporary folder
when doing visual QA.

## Keiko

The catalog contract is in `keiko-catalog.toml`. A release provides the signed `TypeDojo.app`, an architecture-specific ZIP and `SHA256SUMS.txt`. Keiko verifies the checksum, bundle identifier and code signature before installation. Publication order is TypeDojo first, then its Keiko catalog update.

See [docs/design.md](docs/design.md) for the architecture and failure states.

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md) for local setup, contribution paths, validation and pull requests. Bug reports, teaching improvements and focused code changes are welcome.

## References and license

The explanations and exercises are original. Technical reference:
[TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html),
[compiler options](https://www.typescriptlang.org/tsconfig/), and
[Node permissions](https://nodejs.org/api/permissions.html).

MIT OR Apache-2.0. Bundled fonts are OFL; see [ATTRIBUTION.md](ATTRIBUTION.md).

The [editor assistance contract](docs/editor-assistance.md) describes snapshot protection, limits and tool provenance. The editor is a focused two-file learning workspace; completion does not add arbitrary package installation or a complete LSP environment.
