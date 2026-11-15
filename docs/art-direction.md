# Indigo Courtyard — TypeDojo visual direction

## Council and decision

Three read-only advisors ran through T3's provider adapters:

- Claude Fable 5.2 (`claude-fable-5-2`, max): hierarchy, typography, navigation and original crest language.
- Claude Opus 5 (`claude-opus-5`, max): native asset constraints, paper contrast, provenance and editor calm.
- GPT 5.6 Sol (`gpt-5.6-sol`, max): consistency of the learning workflow and compact-window behavior.

The parent selected **Indigo Courtyard**, a paper architectural study garden with a bracket seal.
Fable preferred neutral sumi and a flatter resist-dye visual vocabulary; Opus also preferred flatter
chalk artwork. Sol supported the paper-relief courtyard. The chosen relief art is retained as one
contained masthead, with quieter code-native chapter crests. There are no six large textures or
background artwork under code. This keeps the art visible without sacrificing the workbench.

Experience-first changed the lesson anatomy: the challenge moved beside the editor, rather than
requiring a scroll through theory. The reading panel uses warm paper and ink; the editor uses a
quiet dark surface. Navigation is flat with distinct selection and pass labels. At compact widths,
the learning path is an overlay rather than a squeezed sidebar. Title, challenge and completion
remain actual application text, never baked into an illustration.

Typography: Space Grotesk Bold for identity/titles, Inter for reading, JetBrains Mono for code and
metadata. The bracket-and-slash seal and six chapter crests are original native geometry. The app
icon is rendered from the same bracket vocabulary by scripts/icon.swift.

## Assets and provenance

- `assets/art/courtyard-v1.png`: unmodified ImageGen master, 2188×718.
- `assets/art/courtyard-banner-v1.png`: 960-pixel-wide display derivative, resized with macOS sips.
  Only this ~434 KB image is embedded. There is no startup resampling or oversized texture upload.
- `src/design.rs`: tokens, seal, chapter crests, image upload, edge blending and cached code colors.
- `scripts/icon.swift`: original vector-style app-icon drawing.
- `assets/fonts/OFL-SpaceGrotesk.txt`: additional font license.

Generated with the built-in ImageGen tool, not the CLI. Original prompt:

> Create one original premium editorial illustration asset for TypeDojo, a calm native TypeScript learning application. Wide panoramic composition about 3:1, opaque background solid very dark midnight indigo #101924 blending seamlessly at all edges. Subject on right two thirds: an exquisite miniature architectural study courtyard assembled from folded ivory paper and lapis/cobalt blue geometric blocks, interlocking abstract bracket-shaped archways, six quiet stepping stones progressing across a blue ink garden, a tiny amber bookmark-like accent. Japanese paper craft and contemporary architectural model aesthetic, hand-crafted textured paper edges, precise restrained geometry, softly lit ivory faces, deep navy shadows, subtle woodblock-print grain, exceptionally tasteful editorial art, not glossy 3D plastic, not a game scene. Left third almost empty midnight negative space suitable for separate UI text. No letters, words, code, brands, logos, people, torii religious symbols, watermarks, gradients or neon. Designed to look beautiful at 640x210px in a desktop study app. Raster illustration, not a UI mockup. Save the generated asset as a local file and return its local path as metadata.

## Validation

- Native wide and compact frames inspected. Contrast mistakes in the first paper-panel draft were corrected.
- Native editing/paste with Unicode and Cmd+Enter checked in a separate preview bundle and isolated data folder.
  The compiler result and exact-draft pass state were preserved.
- Format, clippy with warnings denied and locked debug/release builds run.
- No curriculum, compiler contracts, progress schema or learner data changed.
- No new automated UI tests, commits, push or publication in this visual pass.
- An early QA build exceeded egui's texture-size limit; the pre-sized display derivative fixes the cause.

Not yet checked on a physical non-Retina display. Code colors are a small presentation lexer,
not language-server diagnostics or a full TypeScript parser.

## Final council follow-up

Fable's final review identified an inert wide-layout Start practice button. It now requests focus
on the exact lesson editor and still switches compact tabs. Other corrections: left-aligned lab
text/diagnostics, stronger paper-button contrast, disabled scroll-edge shading, rounded artwork
corners, aspect-preserving UV crop, and a pixels-per-point key for the syntax-layout cache.
These were fixed by the parent rather than starting more reviewers. Compact overlay selection and
Learn → Practice were exercised in the native app. Fresh wide/compact captures followed the fixes.

The installed app was updated with its prior bundle preserved in target/install-backups/. Existing
selected lesson, source and exact-draft pass were observed after reopening. No progress migration.
