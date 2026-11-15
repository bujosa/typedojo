# Learn, guided practice, independent lab

The learner first understands terms and syntax, traces a worked example, predicts an outcome,
then solves a small guided exercise before transferring the idea to the existing independent lab.
All 53 lessons gain a structured guide. Technical copy and exercises remain in English.

- Guide content is separate from the stable lab contracts. Every guide defines vocabulary,
  an outcome, prerequisites, a worked example with explanations, a misconception check,
  a smaller practice contract, hints and transfer criteria. No copied third-party prose.
- Guided drafts and exact-source pass evidence are saved separately from lab drafts using
  serde-defaulted fields in progress schema 1. Older app versions ignore and may drop these new fields on save; installation keeps a local backup. Existing course completion remains lab-only.
  Reading, answering a prediction, viewing a solution, running or formatting never credits a lab.
- One native workspace edits either the guided practice or independent lab; switching stages
  cancels its running session. A task panel remains visible beside code on wide screens.
- Prettier 3.6.2 standalone + TypeScript/Estree are pinned and bundled with license/SHA256.
  The background adapter uses fixed options, reads no learner config, executes no learner code,
  runs with Node permission restrictions, and has source/output/time limits.
- Format targets the active file and applies only if its lesson/stage/source snapshot still
  matches. Syntax errors, process errors or edits during formatting preserve current code.
  Formatting changes exact-source completion to pending until checked again. Undo remains available.
- The bundled reference snippets, practice starters/solutions and lab code fields are formatted
  with the same Prettier options. No stored learner draft is rewritten during upgrade.

Validation: real Prettier idempotence/errors, strict tsc reference examples and guided contracts
(starter/reference/alternative/missing/wrong/negative), existing lab contracts, storage compatibility,
then native wide/compact Learn → Practice → Lab and formatter interactions. No automated UI tests.

References: https://prettier.io/docs/browser, https://prettier.io/docs/options,
https://www.typescriptlang.org/docs/handbook/2/everyday-types.html.

Format uses Cmd+Shift+I on macOS. The command modifier prevents Option-key text composition
from inserting a character while invoking formatting. Prettier formatWithCursor remaps saved
breakpoint lines; source and breakpoint snapshots must both match before a result is applied.

## Local validation completed

- `cargo fmt --check`, Clippy with warnings denied, and 14 Rust/real-runtime tests passed.
- Original lab verification: 53 × 6 contract cases passed after formatting. One recursive-tree
  expected-error assertion was moved to the property line where TypeScript reports the error.
- Guided verification: 53 × 6 contract cases passed; all 53 worked examples compiled and all
  53 guided playgrounds ran. Repository format check: 679 code fields, zero differences.
- Native macOS review used separate `TYPEDOJO_HOME` folders, never learner data. Wide and
  820×700 layouts were inspected. Predictions explain both wrong and correct choices.
- Verified Format button, Cmd+Shift+I, undo, syntax-error preservation, active-playground isolation,
  stage/draft separation and persistence across restart. A delayed Node wrapper demonstrated
  that a newer edit survives a stale formatter result. Guided completion left the course count unchanged.
- Verified guided failure includes a next-step hint; independent lab success still requires its
  own original contract. No automated UI test suite was introduced.

Formatter replacements now create explicit undo boundaries before and after the edit, so immediate
paste → Format → Undo restores the exact unformatted source. Redo and repeated rapid formatting
were verified natively; see the dated E2E report.
