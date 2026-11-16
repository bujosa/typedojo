# TypeDojo roadmap

Goal: a course from first programming steps to confident professional TypeScript, entirely in the
app. Scope is measured by tested labs and projects, not by a claim that one set of short lessons teach
everything.

## 0.1 implemented

- Fifty-three original lessons, six sequential blocks, editor, hints and worked solutions.
- Structured reading for every lesson: outcomes, vocabulary, prerequisite links, traced examples and explained predictions.
- Fifty-three additional guided exercises with real compiler/runtime contracts and separate saved progress.
- Bundled offline Prettier, active-file Format, undo, syntax failure preservation and stale-result protection.
- Real strict compiler contracts, negative assertions and reference-answer verification.
- Runtime examples for twenty function and project labs, including validator edge cases.
- Saved drafts, exact-answer completion, resume, compact and wide layouts, keyboard check.
- Owned compiler folders, deadlines, bounded output, isolated runtime contexts, no package download.
- Native two-file IDE focus mode: Run, streaming logs, source-mapped V8 debugger, breakpoints,
  step over/into/out, pause/continue/stop, variables, call stack and expression inspection.
- Public macOS Apple Silicon v0.1.0 release and verified installation through Keiko.

## 0.2 implemented

- Bundled, checksum-pinned Node 24.22.3 and TypeScript 5.9.3 for Check, Run, Debug, Prettier and completion. No learner terminal setup.
- Offline semantic member and workspace-symbol suggestions, explicit Suggest/Ctrl+Space, keyboard acceptance and undoable replacement.
- Exact-snapshot background requests, bounded processes, Unicode-safe ranges and a native caret popup.
- macOS 13.5+ is required by the included Node runtime. Developer ID/notarization remains separate release work.

## Complete the beginner path

- Deeper JavaScript: destructuring, module scope, asynchronous iteration,
  modules and import/export; several exercises per concept with escalating difficulty.
- Guided explanations on top of the real debugger for branching and iteration; richer diagnostics.
- Expand the existing study tracker into a multi-file project with additional challenges.

## Complete the TypeScript path

- Deeper inference and contextual typing, structural assignability, excess properties,
  overloads, variance, classes and access modifiers.
- Modules and module resolution, ESM/CommonJS, declaration files, tsconfig and project references.
- More varied labs, asynchronous runtime checks, more varied quizzes, spaced review.
- Project: a validated API client handling failure, cancellation, pagination and retries.

## Professional projects

- Extend the two-file workbench to full projects: folders, more files, test output, fixtures and reset.
- Type-safe domain library, unit/integration testing, package exports and declarations.
- Node API and browser UI projects with runtime schemas, error contracts and accessibility.
- Tooling curriculum: linting, formatter configuration, reproducible builds, CI, dependency maintenance and safe releases.
- Refactoring exercises and code-review challenges; performance of types and compiler diagnostics.
- Practical capstone assessed by behavior, edge cases, types and design decisions, not just compiling.

## Distribution

- Completed: public repository/release and Keiko catalog integration, with anonymous install/update verification.
- Contributor setup, content-authoring guidance and issue/PR templates are available in CONTRIBUTING.md.
- Next: Developer ID signing/notarization needs the maintainer’s Apple account.
