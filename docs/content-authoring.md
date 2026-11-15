# Writing TypeDojo content

Content is original English teaching material in two JSON arrays embedded at build time. Use the existing `values` lesson/guide as a small reference and a relevant function/project lesson as the reference for runtime behavior. Read [the learning contract](learning-experience.md) before editing.

## Keep the two stages aligned

`content/course.json` defines the independent lab. `content/learning.json` defines its reading and guided exercise. The arrays must have the same length, matching IDs and matching order. Prerequisites may only refer to earlier lessons.

A course entry has:

- `id`, `section`, `title`: stable identity and navigation. IDs contain lowercase letters and hyphens only. Do not rename a shipped ID casually: saved drafts/completion use it.
- `concept`, `task`, `hint`: explanation, exact expected exports/behavior and a useful smaller clue.
- `starter`: plausible incomplete or incorrect code that the checker rejects.
- `solution`, `alternative`: independently valid answers that the checker accepts.
- `checks`: TypeScript contract code. The harness imports the answer as `s` and provides `Equal<A, B>` and `Assert<T extends true>`.
- `negative`: an answer the checker must reject. For functional labs, include a wrong implementation with otherwise plausible types.
- `runtime`: JavaScript behavior assertions using the compiled `exports` object, or an empty string for a type-only lesson. State that limitation honestly.

A guide has `id`, `outcome`, `prerequisites`, `vocabulary` (`term`/`meaning` pairs), `example`, an `explanation` array, `prediction`, `practice` and `transfer`.

- Explain vocabulary before using it. Trace a small executable example in at least two explanation steps.
- A prediction has `question`, exactly three plausible `choices`, a zero-based `correct` index and an explanation of the misconception.
- `practice` has its own `task`, `starter`, `solution`, `alternative`, `negative`, `checks`, `runtime`, `hint` and `playground`. Its compiler contract is separate from the independent lab.
- The playground imports from `./answer` and demonstrates the guided solution. Keep it within the documented two-file runtime capabilities.
- `transfer` explains what the learner should carry into the independent task.

## Contract examples

For an exported numeric `attempts`, a type check can be:

```ts
type AttemptsAreNumbers = Assert<Equal<typeof s.attempts, number>>;
```

A runtime assertion can distinguish the correct initial value from a wrong but well-typed answer:

```js
if (exports.attempts !== 1) throw new Error("attempts must start at 1");
```

Check the learning objective, not incidental spacing or the spelling of an equivalent implementation. Use `@ts-expect-error` for intentionally invalid TypeScript; place it on the exact diagnostic line, especially after formatting. Accept genuinely valid alternatives. Hints and worked answers must explain why the solution works without silently crediting completion.

## Validate and review

```sh
export TYPEDOJO_HOME="$(mktemp -d -t typedojo-content)"
node tools/format-content.mjs --write
cargo run --locked -- --verify-content
cargo run --locked -- --verify-learning
node tools/format-content.mjs
cargo test --locked
```

The verifiers reject starters, missing/wrong exports and negative answers, and accept both solutions. Guided verification also compiles the example and runs its playground with the worked solution. Read diagnostics and fix a contract rather than removing a failing case. If adding a lesson, update explicit course-count assertions, related navigation/marketing counts and the roadmap as part of that reviewed change.

Then review Learn → prediction → Guided practice → Independent lab natively in the isolated profile. Check wrong-answer feedback, the hint, successful grading, revisiting saved drafts and keyboard access at wide/compact sizes. A successful Run must not count as a passed lab. Use fictional progress in screenshots.

Do not copy handbook prose or third-party exercises. Link authoritative technical references, write original explanations and include attribution/licenses for new assets. Never run untrusted submitted code on a learner’s real profile or broaden the runtime boundary as a shortcut to a new exercise.
