# Native TypeScript editor

The study workbench now uses the official Tree-sitter TypeScript grammar plus JavaScript highlight
queries instead of a hand-written word scanner. Parsing is presentation only. The unchanged tsc
adapter still decides whether a type contract passes; Node still checks selected runtime examples.

Implementation: `src/editor.rs`. It owns parser/highlighter configuration, cached text layout,
editor chrome, fixed line-number gutter, current-line paint, cursor position and scroll behavior.
The cache keys source and display pixel density. The original art tokens remain in `src/design.rs`.

The code viewport does not wrap source lines. Its numbered gutter stays fixed during horizontal
scroll; vertical scrolling keeps numbers aligned with the same 22pt row metrics as the code.
Plain Enter preserves leading whitespace, adding four spaces after an opening brace/bracket/paren.
Tab/Shift+Tab use egui's existing indentation handling. Undo remains native TextEdit input history.

Diagnostics are compiler output, not simulated inline warnings. Clicking an `answer.ts` diagnostic
reveals its source line, scrolls toward it and focuses the editor. `contract.ts` errors remain labeled
contract checks. Solutions use the same syntax palette in a selectable, read-only code view.
The initial editor pass did not include a debugger. The subsequent IDE engine adds a two-file
workspace and real debugging; see [ide-engine.md](ide-engine.md). A language server and semantic
autocomplete remain outside the current implementation.

Malformed or unfinished code remains editable. A highlighting configuration error falls back to
plain text with an explicit notice. Highlighting never changes source or completion state.

Validation on macOS Apple silicon:

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, locked debug/release builds.
- Native isolated QA: multiline comments, generics, functions, parameters, template interpolation,
  multiline paste, automatic indentation, undo, vertical scrolling, horizontal cursor reveal with
  a fixed gutter, empty draft, and Ln/Col updates.
- A real TS2322 error was clicked: cursor moved from line 4 to line 2. Corrected code passed the lab.
- Read-only solution colors and the 820×700 Practice layout inspected; the Check action was moved
  above the editor so it remains reachable before scrolling through code.
- Compiler/course/progress schema unchanged. Their previous full course verification was not
  rerun for this presentation-only change. No automated UI tests added.

The previous art changes were already dirty in the original checkout. This work is isolated in
`bujosa/typedojo-editor-polish` with a copy of that visual baseline; the original checkout is intact.

Source references:
- https://github.com/tree-sitter/tree-sitter-typescript
- https://github.com/tree-sitter/tree-sitter-javascript
- https://github.com/tree-sitter/tree-sitter/tree/v0.25.10/highlight

MIT notices for the grammar/runtime and existing font licenses now ship in app Resources/Licenses.
