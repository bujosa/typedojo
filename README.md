# TypeDojo

A native TypeScript practice app in the Keiko family. Read an explanation, solve a lab in the built-in
editor, check with the real TypeScript compiler, and resume your saved draft next time.

**0.1 is an initial course, not a complete professional qualification.** Fifty-three lessons introduce
JavaScript basics and progress to advanced type transformations and professional API patterns.
Every lesson has original theory, a task, a hint, a worked solution, and compiler contracts.
Twenty function and project labs also check runtime examples. Further projects and coverage are tracked in
[ROADMAP.md](ROADMAP.md).

## Start

macOS, Rust (pinned by `rust-toolchain.toml`), Node.js 22.13+, and TypeScript 5.9+ on PATH:

```sh
npm install -g typescript
cargo run
```

You do not need a database, Docker, a server or an account. Once the tools are installed, labs work
offline. TypeDojo never installs npm packages automatically.

- Choose a lesson in the left rail; search by title or section.
- Read **Learn**, then edit **Practice** (side by side in a wide window).
- **Check lab** / **Cmd+Enter** runs strict compiler checks and, for selected function labs, runtime examples.
- Compiler diagnostics name `answer.ts` (your code) or `contract.ts` (the required public contract).
- A pass applies to the exact checked answer. Editing it makes the lab pending again.
- Hints and worked solutions do not automatically complete a lab.
- Drafts and the selected lesson are saved locally; save errors are shown and can be retried.

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

Compiler work runs in disposable owned folders, has a 20-second deadline, and never compiles your
existing projects. Runtime labs use a separate Node process (96 MiB heap, four-second deadline),
a VM without host APIs, disabled string/Wasm code generation, and Node permission flags allowing
reads only in that temporary folder. This limits mistakes; **Node VM and permission flags are not
a security sandbox for malicious code**. Use only your own trusted practice answers.

`TYPEDOJO_TSC` and `TYPEDOJO_NODE` can select absolute executable paths when launching from Finder.
Homebrew's standard binary directories are discovered automatically.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --verify-content
python3 scripts/bundle.py  # build, sign ad-hoc, zip, SHA256SUMS.txt
```

Content verification uses the actual compiler for every starter, reference solution, alternative,
missing exports, wrong exports and negative answer. Functional negatives include implementations
whose signatures are correct but whose behavior is wrong.

`TYPEDOJO_SCREENSHOT=/tmp/typedojo.png` captures a native frame and closes the app;
`TYPEDOJO_WINDOW=820x700` checks the compact layout. Always set `TYPEDOJO_HOME` to a temporary folder
when doing visual QA.

## Keiko

A draft catalog entry is in `keiko-catalog.toml`. Add it to Keiko only after publishing a verified
signed release with the exact bundle identifier, archive name and checksum asset. This project
has not been made public automatically.

See [docs/design.md](docs/design.md) for the architecture and failure states.

## References and license

The explanations and exercises are original. Technical reference:
[TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html),
[compiler options](https://www.typescriptlang.org/tsconfig/), and
[Node permissions](https://nodejs.org/api/permissions.html).

MIT OR Apache-2.0. Bundled fonts are OFL; see [ATTRIBUTION.md](ATTRIBUTION.md).
