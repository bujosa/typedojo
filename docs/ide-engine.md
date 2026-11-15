# Native IDE workbench

The learner edits answer.ts and a saved per-lesson playground.ts, runs the playground,
reads real console events and exceptions, and debugs the exact source snapshot. Check lab
remains the only completion authority. Run success never credits a lesson.

Focus uses a compact command bar, file explorer/debug sidebar, central editor, resizable
bottom Console/Problems panel and status bar. The lesson task is available on demand.
There is no lesson artwork or long challenge text consuming editor space in this mode.
Compact windows retain commands, file tabs and the bottom panel; the sidebar can be hidden.

Runtime boundary: Rust owns compilation, processes, cancellation, deadlines and temporary
files. A trusted Node bridge uses an inspector Session in a worker connected to the main
thread: no TCP inspector port, webview, shell, downloaded packages or exposed terminal.
TypeScript compiles both files with strict options and source maps. Only the compiled
answer module can be imported by the playground. Learner code runs in a VM context without
process, filesystem, network, host require or timers. Node permission mode prohibits
inspector sessions, so this new interactive workbench runs without that process flag.
This is educational isolation for locally authored code, not a security boundary for
hostile code (as documented by Node for vm). Existing Check lab runtime tests retain
their separate permission-mode runner. No machine-wide security setting is changed.

The bridge translates original-source breakpoints to generated locations and maps paused
frames back to TypeScript. Debug starts paused at the first executable playground statement.
Continue, pause, step over/into/out, frame selection and expression evaluation use real V8
inspector commands. Scope inspection reads descriptors without invoking getters.

States: idle, compiling, running, paused, stopping, finished, failed. Controls derive from
these states. Editing is locked for the session snapshot. Switching lessons or closing the
application cancels the session and reaps the child. Compilation is capped at 20 seconds,
active execution at 5 seconds, total debugging at 10 minutes. Output and source sizes are
bounded. Missing tools, compile failures, thrown exceptions and deadlines are explicit.

Progress schema 1 gains optional playground and breakpoint fields using serde defaults;
existing answers, selected lesson and pass evidence remain compatible. Existing releases
ignore these additional fields. A rollback can discard new playground fields on its next
save; backups of the previous app remain available. No remote publication is part of this work.

Protocol references:
- https://nodejs.org/api/inspector.html (worker connectToMainThread)
- https://nodejs.org/api/vm.html (VM isolation limits)
- https://chromedevtools.github.io/devtools-protocol/tot/Debugger/
- https://chromedevtools.github.io/devtools-protocol/tot/Runtime/

Validation covers the real tsc/Node bridge: logs and exports, compilation errors, exceptions,
source-mapped breakpoints, stepping, variables, evaluation, cancellation and loop deadlines.
UI validation uses the native application in wide and compact windows.

## Observed validation (15 November 2026)

- Engine tests use the real local TypeScript 5.9.2 compiler and Node 24.1.0. They cover logs,
  compile/runtime errors, original-source breakpoint binding, duplicate entry breakpoints,
  call frames/variables, side-effect-free evaluation, over/into/out stepping, pause while
  running, stop while paused, execution deadline and the 500-message console cap.
- Storage tests preserve old schema-1 drafts and persist playgrounds/breakpoints without
  invalidating unchanged answer completion.
- Native wide QA: Run logged 42, Debug stopped at answer.ts:3 with n=21/value=42,
  evaluation returned 63, step over advanced to line 4, Continue completed, compiler
  TS2322 linked to line 2. This used disposable TYPEDOJO_HOME data.
- Native 820×700 QA: command wrapping, debugger sidebar, source highlight, stop, Task
  dialog and Check lab (including actual runtime examples) verified; only Check lab
  changed completion from 0/53 to 1/53. Running snapshot ignored paste attempts.
- Expand and Restore visibly resize the console/editor allocation. Native automated drag
  did not move the divider; free dragging remains unverified. Expand/Restore is the
  documented and verified size control.
- `cargo run --locked -- --verify-content` passed all 53 lessons × 6 cases (318 cases).
  No automated UI tests were added.
