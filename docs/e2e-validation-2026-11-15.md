# Native E2E validation — 15 November 2026

Status: TypeDojo curriculum and listed application flows passed. One TypeDojo defect was fixed.
Keiko and PodDojo received integration and smoke coverage; the entire published ecosystem and
cluster curriculum are **not** certified as fully tested by this run.

## Environment and isolation

macOS Apple silicon; native Rust/egui apps. All interactions used the native computer-use API,
not a browser, simulated UI state or a new committed UI test suite. Test app copies have separate
bundle identifiers. Every running test instance used a temporary TYPEDOJO_HOME, PODDOJO_HOME or
KEIKO_HOME. Keiko used its supported KEIKO_CATALOG and KEIKO_APPS_DIR overrides with copied,
signed QA bundles. Production catalogs and repositories were not published or changed.
The real TypeDojo learner progress remained byte-for-byte unchanged throughout the test run.
The source and logs live in the isolated bujosa/typedojo-e2e worktree.

## TypeDojo course traversal

All 53 lessons were selected through Next lesson and the actual application UI. For every lesson:

1. Confirmed outcome, defined terms and worked example are present.
2. Selected all three prediction choices and verified one correct and two incorrect explanations.
3. Checked the guided starter and confirmed rejection with no independent lab credit.
4. Opened the guided worked solution; viewing it did not grant completion. Pasted its visible
   source in the editor, checked it successfully and ran the saved playground.
5. Continued to the original independent lab, checked its starter (rejected), opened/pasted its
   visible worked solution, checked it (passed) and ran its playground.
6. Verified the global counter advanced only for the independent lab, then used Next lesson.

Totals: 53 guides, 106 rejected starters, 106 accepted solutions, 106 successful UI runs and
159 prediction choices. Final course state was 53/53. After restart in an 820×700 window, all
original lab passes and guided passes were present. The persisted snapshot was compared with
all 106 reference sources and exact-source pass evidence. Subsequent negative IDE scenarios
intentionally edited the first guided practice; the earlier complete-course snapshot is retained.

## IDE and failure flows checked

- Wide and compact reading/workbench layouts; focus/study views; task panel/dialog and hints.
- Search with no results, Clear search and lesson selection; scoped reset cancellation and
  confirmation; guided reset leaves lab credit intact; independent reset invalidates its pass.
- Cmd+Enter checks; command palette search, empty state and starting Debug with keyboard.
- Real output, warning and error console messages, severity filtering, copy acknowledgement,
  Expand/Restore; Find with a match, no match, next/previous and Escape; editor font size/reset.
- Debug pauses at entry; Step over, Step into and Step out retain original TypeScript locations.
  Stack frames, arguments and local values were inspected. `value + 1` evaluated to 4; assigning
  `value = 99` was rejected as a side effect and program output remained 6.
- Gutter breakpoints in both files bound and stopped on the requested source lines. Continue
  resumed between them. Pause interrupted a running infinite loop; Stop reaped the session.
  Leaving focus stopped a paused session. The active-time limit also terminated infinite loops.
- Compiler failure appears under Problems; clicking a diagnostic opens its file. Runtime errors
  show their mapped TypeScript location. These failures do not credit completion.
- Format button and Cmd+Shift+I; syntax-error preservation; independent active-file formatting;
  immediate paste/Format/Undo, Redo and a second rapid edit. A delayed real Node formatter
  demonstrated that a newer edit is preserved and a stale result is discarded.
- Missing Node and TypeScript show recovery instructions while preserving the source and count.
  Relaunching with tools available allowed the same draft to pass its guided compiler/runtime check.
- Malformed progress opens in saving-unavailable practice mode and remains byte-for-byte intact.
  A second real app instance cannot acquire an active folder lock; the first instance retains its
  pass evidence and releases the lock on graceful close.

Native automation needed refreshed accessibility snapshots around asynchronous transitions;
initial inconclusive Pause clicks were repeated with visible controls and then passed.

## Fixed defect TD-E2E-001

Before: pasting code and immediately formatting could merge both edits into egui's timed undo
batch, so Cmd+Z returned to the draft before the paste. After: applying a formatter result records
explicit pre-format and post-format history points for the active editor, including cursor state.
Obsolete redo branches are cleared through egui's undoer API. Immediate Undo, Redo and repeated
formatting were verified in the rebuilt native app. Static checks and all 14 existing tests passed.

## Keiko and PodDojo coverage

- Keiko `cargo xtask ci`: passed.
- Keiko `cargo test -p keiko-native -- --ignored`: both real network tests passed — install/update
  PodDojo, preserve the installed bundle when an asset is not listed in checksums, and Keiko
  self-update replacement in temporary folders. No installed user application was replaced by tests.
- Keiko UI: supported custom catalog detected two signed isolated app copies; Open invoked each
  already-running QA app. Grid/list views, no-result search and the empty Updates filter were checked.
- PodDojo `cargo xtask ci`: passed, including validation of 40 lessons, 190 quiz questions,
  159 lab steps and 68 mock tasks. This is content validation, **not** execution of every lab/task.
- PodDojo UI: created E2E Learner in an isolated data folder, opened the first lesson, submitted
  five correct and one incorrect quiz answers, saw score 5/6 and explained feedback. Home, Learn,
  Review, Exams, Reference, Docker explorer, Lab cluster, Progress and Setup navigation passed.
- PodDojo `dojo verify docker-terminal-yaml-vim`: all four local-file lab steps and 13 extra
  negative/alternative cases passed in another temporary folder. No cluster resource was changed.

## Remaining findings and coverage limits

| ID | Status | Finding |
|---|---|---|
| TD-E2E-001 | Fixed and retested | Immediate-format Undo grouping described above. |
| K-E2E-001 | Open, medium | Several custom-painted Keiko controls, including Open, appear as unknown accessibility nodes rather than named buttons. Coordinate operation works; accessibility needs improvement. |
| K-E2E-002 | Open, low | Updates says Everything is up to date even when a test dojo's latest release is unavailable. The individual TypeDojo row correctly shows Installed and no latest version. |
| DIST-001 | Not covered | TypeDojo latest-release API returned HTTP 404 for the current identity. Local detection/open does not prove published release install/update. |
| POD-ENV-001 | Not covered | Existing fixed-name PodDojo cluster and resources belong to learner work. Full Docker/CKAD labs and mock exams were not reset or executed against them. Setup also reports kubectl 1.34.1 vs cluster 1.37.0; use an isolated compatible environment for that sweep. |

The run does not cover notarization/Gatekeeper on a fresh Mac, other operating systems, live
package upgrades, all filesystem-permission failures, or hostile-code security guarantees.

## Evidence and exact commands

Local artifacts are in target/e2e/: curriculum-matrix.json, progress-after-course.json,
results.json and command logs. These contain fictional test code and progress only.

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
# Exact tested native app executable, separately:
<qa-app>/Contents/MacOS/typedojo-e2e --verify-content
<qa-app>/Contents/MacOS/typedojo-e2e --verify-learning
node tools/format-content.mjs
python3 scripts/bundle.py
# Keiko:
cargo xtask ci
cargo test -p keiko-native -- --ignored
# PodDojo:
cargo xtask ci
PODDOJO_HOME=<temporary-folder> target/release/dojo verify docker-terminal-yaml-vim
```

TypeDojo tests: 14 passed. Original and guided compiler verification: 318 cases each, 636 total.
All 53 worked examples compiled and all 53 guided playgrounds ran in learning verification.
All 679 bundled code fields match the pinned Prettier formatter. Bundle signing and ZIP checksum
verification passed. No feature was pushed, merged, published or deployed by this task.

## Local delivery

The corrected signed bundle was installed locally after backing up the prior application and
progress under target/install-backups/. The installed application reopened on the learner's
selected fallback lesson with 1/53 completed labs. The learner progress still matched both the
pre-test hash and the installation backup byte-for-byte. All QA applications were closed.
