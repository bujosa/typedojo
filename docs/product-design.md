# TypeDojo product design refinement

Applies the product-workflow-design and ui-visual-validation skills to the existing Indigo Courtyard
identity. The user journey remains choose/resume → understand → edit → check → review → continue.
This pass changes presentation and navigation; curriculum, compiler contracts and progress schema
remain unchanged.

## Decisions

- Chapter groups replace the flat 53-item rail. The active chapter opens on entry, each group can
  expand/collapse, and its count/bar derives from exact-answer passes. Search spans every chapter.
  Search results can open the whole chapter. No-results has an explanation and Clear search action.
- The wide workspace gives about 42% to reading (capped at 520pt) and the rest to code. Article
  leading is 22pt and both reading and practice scroll independently.
- A smaller masthead preserves the artwork while returning height to the task. Compact Practice
  uses an 82pt masthead and a segmented Learn/Practice control.
- Focus on code hides the reading pane and rail and increases code viewport height. Study view
  restores the previous rail preference; changing view never changes the draft or pass state.
- Source syntax, line numbers and compiler diagnostics retain the existing IDE-style workbench.
  Result cards distinguish verified success, feedback and unchecked work with explicit text.
- Worked solutions and reset confirmation use centered, styled modal cards with a backdrop. Escape
  closes them. Cmd+Enter cannot start a background check through an open modal. Copy code has an
  acknowledgement; resetting still requires the explicit confirmation and affects one answer only.
- The compact chapter drawer uses the same panel, border, type and Close control as the other UI.

## Verification

Native macOS isolated QA at 1440×900 and 820×700:

- Expanded/collapsed chapters, selected another chapter's lesson, switched Focus/Study view.
- Entered a correct generic type; real compiler passed and chapter progress changed to 1/16.
- Opened worked solution; confirmed colors/read-only code and modal/backdrop layout.
- Cmd+Enter while modal open did not start a check; Escape closed it.
- Cancelled reset with a passed answer; the answer and pass remained.
- Searched for an absent term and recovered with Clear search.
- Inspected compact Practice: primary Check action, reduced artwork masthead and editor remained
  visible; inspected the compact chapter drawer and solution modal.

Static gates: cargo fmt --check; cargo clippy --all-targets -- -D warnings; locked debug/release
builds and signed bundle generation. No automated UI tests were added. No new grading rules or
content changes, so the previous full content verification was not repeated for this visual pass.

This checkout is bujosa/typedojo-product-design, isolated from the existing dirty art/editor checkouts.
The resulting bundle is installed locally with the prior app retained under target/install-backups.
No commit, push, PR or public release is part of this pass.

## IDE focus revision

Focus now opens the native IDE workbench instead of the enlarged lesson layout described above.
Its compact toolbar, two-file editor, resizable console and debugger replace the masthead/challenge
stack in Focus. The learning path and reading view remain available. See [ide-engine.md](ide-engine.md).
