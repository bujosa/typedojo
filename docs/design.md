# TypeDojo 0.1

A learner starting with programming fundamentals selects a lesson, reads an original short explanation,
edits its starter, and checks it with the real TypeScript compiler. Compiler diagnostics are evidence;
merely clicking Read or revealing a solution never earns completion. Reopening restores drafts and
the selected lesson. Editing a passed solution makes it pending until checked again.

This is a separate native application, following Keiko/PodDojo's Rust + egui architecture. The
curriculum is embedded, the model is independent from UI, and compiler work runs on background
threads. Every lab checks type contracts; twenty function and project labs also execute sample inputs in a
restricted Node VM. No packages are downloaded and no learner project is imported.

States: not checked, checking, compiler unavailable, incorrect, passed, storage failure. A result is
associated with its exact lesson and source; navigation and edits cannot credit another draft.
Progress is JSON with an explicit schema version, written via rename. Unreadable or newer state is
preserved and disables writes until the learner resolves the problem. Multiple simultaneous app
instances are rejected by the data-folder lock. Crashed instances require manual lock removal.

The compiler works in an owned temporary directory with a self-contained tsconfig (`strict`,
`types: []`, ES2022). Output goes to a file to avoid pipe deadlocks and is bounded when
read. Compiler jobs time out. An absolute compiler path may be supplied with TYPEDOJO_TSC.
Compiler directives in the editor are rejected to prevent accidentally disabling checks. This is
local educational feedback, not an anti-cheat system or a sandbox for hostile imports.

Initial scope: fifty-three lessons from annotations through a typed API contract, each with contracts,
negative assertions, hints and worked solutions. No claim of a full TypeScript course, certification,
autocomplete, multiple profiles or spaced repetition in 0.1.

Keiko integration is staged in `keiko-catalog.toml`; it must not be added to the production catalog
until a signed release and checksums can actually be downloaded. Existing apps remain unaffected.

## Interactive engine

The IDE focus workbench has a separate run/debug session owner (`src/engine.rs`), native
presentation (`src/ide.rs`) and trusted Inspector bridge (`runtime/bridge.cjs`). It never credits
completion or changes the existing lab checker. See [ide-engine.md](ide-engine.md) for its lifecycle,
source maps, local trust boundary, deadlines and progress compatibility.
