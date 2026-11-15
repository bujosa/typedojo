# Workbench interaction refinement

The actor is a learner editing a local TypeScript lab. The next action should stay clear while
reading code, running an experiment, inspecting a pause or correcting a compiler error.
This refinement keeps the existing native architecture, Indigo Courtyard identity, local data
and real compiler/debugger engine. It adds interaction tools rather than another dashboard.

- A searchable command palette exposes the actual available actions and their shortcuts.
  Cmd+Shift+P opens it; Up/Down, Enter and Escape work without a mouse. Commands unavailable
  in the current execution state remain disabled. A modal palette blocks global Check shortcuts.
- Find in file (Cmd+F) searches literal, case-sensitive text in the active file. It reports real
  match counts and selects/reveals the next/previous occurrence. It does not edit the source.
- View offers editor text sizes from 11 to 22 px and Reset. Syntax layout, scrolling, line
  numbers, breakpoint hit areas and paused-line paint use the same scaled row height.
- The console has text search, All/Output/Warnings/Errors categories, visible/total counts,
  Follow, Copy visible logs with acknowledgement, and explicit empty/filter-empty states.
  Timestamps are arrival times measured from the actual session start, including paused time.
- Ready/Compiling/Running/Paused/Finished/Failed represent the runtime, independently of lab
  completion. Spinners appear only during active work. Source tabs use flat surfaces and a
  selection rule; the code surface avoids nested card borders.
- Explorer visibility, font size and output height are retained when switching lessons in
  the same app session. Lesson-specific logs, execution and search context are reset.
- Expand/Restore remain available for output sizing, and View exposes an explicit output-height control. The unverified free-drag grip is removed.

Scope: native UI only. No curriculum, grading, execution permissions, subprocess contracts,
progress schema or network behavior changes. No automated presentation tests are added.
Validation uses disposable TYPEDOJO_HOME folders in wide/compact native windows plus the
formatter, Clippy and locked builds. Existing backend tests remain intact.

## Native validation

- Wide QA: Cmd+Shift+P focused search; filtering for file then Down/Down/Enter opened
  playground.ts. Real Run produced logs/warnings/errors. Errors showed 1/6 logs, Copy
  acknowledged, text filter showed 0/6 and Clear filters restored 6/6.
- Compact 820×700: Find highlighted all three message occurrences, selected the current
  one and reported its real line/column. Enter moved forward; Shift+Enter moved backward.
  Escape closed it. Text size 16 px kept line numbers and breakpoints aligned.
- Compact debugger: real breakpoint, variables and source highlight remained usable with
  larger text. Palette disabled Run/Check while paused and Stop ended the process.
- No-matching-command recovery and Escape verified; Cmd+Enter through an open palette
  did not start a lab check or change completion.
- Static validation: cargo fmt --check, cargo clippy --all-targets -- -D warnings and
  cargo build --locked. This pass changes no grading/runtime/storage logic, so prior
  engine/course verification is retained rather than repeated for presentation work.

- View output-height control was set from 190 to 280; the console visibly enlarged and the
  editor allocation shrank accordingly. The free-drag affordance is no longer exposed.
