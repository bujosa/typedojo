//! Lesson artwork is decoded off-thread and cached by stable lesson identity.
use egui::{ColorImage, TextureHandle};
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc,
};
const IMAGES: &[(&str, &str, &[u8])] = &[
    (
        "values",
        "Names bind to typed values",
        include_bytes!("../assets/art/values-v1.png"),
    ),
    (
        "arrays",
        "Order gives each item a position",
        include_bytes!("../assets/art/arrays-v1.png"),
    ),
    (
        "objects",
        "Fields describe one structured value",
        include_bytes!("../assets/art/objects-v1.png"),
    ),
    (
        "functions",
        "Inputs and outputs form a contract",
        include_bytes!("../assets/art/functions-v1.png"),
    ),
    (
        "branches",
        "Narrow the possibilities before acting",
        include_bytes!("../assets/art/branches-v1.png"),
    ),
    (
        "loops",
        "Repeat work and return a result",
        include_bytes!("../assets/art/loops-v1.png"),
    ),
    (
        "discount",
        "Calculate once; reuse the rule",
        include_bytes!("../assets/art/discount-v1.png"),
    ),
    (
        "eligible",
        "Combine conditions deliberately",
        include_bytes!("../assets/art/eligible-v1.png"),
    ),
    (
        "initials",
        "Normalize text before composing it",
        include_bytes!("../assets/art/initials-v1.png"),
    ),
    (
        "fallback",
        "Missing differs from empty",
        include_bytes!("../assets/art/fallback-v1.png"),
    ),
    (
        "greeting",
        "Default only what is omitted",
        include_bytes!("../assets/art/greeting-v1.png"),
    ),
    (
        "doubled",
        "Transform each item independently",
        include_bytes!("../assets/art/doubled-v1.png"),
    ),
    (
        "passing",
        "Keep the items that match",
        include_bytes!("../assets/art/passing-v1.png"),
    ),
    (
        "count-tags",
        "Combine many values into one result",
        include_bytes!("../assets/art/count-tags-v1.png"),
    ),
    (
        "counter",
        "State can live behind a function",
        include_bytes!("../assets/art/counter-v1.png"),
    ),
    (
        "safe-divide",
        "Handle failure as part of the result",
        include_bytes!("../assets/art/safe-divide-v1.png"),
    ),
    (
        "optional",
        "A field may be absent",
        include_bytes!("../assets/art/optional-v1.png"),
    ),
    (
        "literals",
        "Known values define a closed set",
        include_bytes!("../assets/art/literals-v1.png"),
    ),
    (
        "tuples",
        "Position gives each element meaning",
        include_bytes!("../assets/art/tuples-v1.png"),
    ),
    (
        "readonly",
        "Protect the shape from mutation",
        include_bytes!("../assets/art/readonly-v1.png"),
    ),
    (
        "unknown",
        "Prove what an unknown value contains",
        include_bytes!("../assets/art/unknown-v1.png"),
    ),
    (
        "results",
        "Each outcome carries its own data",
        include_bytes!("../assets/art/results-v1.png"),
    ),
    (
        "generics",
        "One container preserves many types",
        include_bytes!("../assets/art/generics-v1.png"),
    ),
    (
        "generic-functions",
        "Let the input determine the output",
        include_bytes!("../assets/art/generic-functions-v1.png"),
    ),
    (
        "constraints",
        "Accept only shapes that fit",
        include_bytes!("../assets/art/constraints-v1.png"),
    ),
    (
        "keyof",
        "Keys define the permitted choices",
        include_bytes!("../assets/art/keyof-v1.png"),
    ),
    (
        "mapped",
        "Transform every field consistently",
        include_bytes!("../assets/art/mapped-v1.png"),
    ),
    (
        "utilities",
        "Reuse the shape; change its contract",
        include_bytes!("../assets/art/utilities-v1.png"),
    ),
    (
        "intersections",
        "Combine independent requirements",
        include_bytes!("../assets/art/intersections-v1.png"),
    ),
    (
        "interfaces",
        "Describe a structural agreement",
        include_bytes!("../assets/art/interfaces-v1.png"),
    ),
    (
        "typequeries",
        "Derive types from existing values",
        include_bytes!("../assets/art/typequeries-v1.png"),
    ),
    (
        "records",
        "Map known keys to their values",
        include_bytes!("../assets/art/records-v1.png"),
    ),
    (
        "required",
        "Make every required field explicit",
        include_bytes!("../assets/art/required-v1.png"),
    ),
    (
        "satisfies",
        "Check compatibility; retain precision",
        include_bytes!("../assets/art/satisfies-v1.png"),
    ),
    (
        "recursive",
        "A structure can contain itself",
        include_bytes!("../assets/art/recursive-v1.png"),
    ),
    (
        "extract",
        "Select the members that match",
        include_bytes!("../assets/art/extract-v1.png"),
    ),
    (
        "parameters",
        "Derive a function's argument contract",
        include_bytes!("../assets/art/parameters-v1.png"),
    ),
    (
        "promises",
        "A result arrives later",
        include_bytes!("../assets/art/promises-v1.png"),
    ),
    (
        "conditional",
        "Choose a type with a condition",
        include_bytes!("../assets/art/conditional-v1.png"),
    ),
    (
        "infer",
        "Recover the type inside a pattern",
        include_bytes!("../assets/art/infer-v1.png"),
    ),
    (
        "templates",
        "Compose new names from known pieces",
        include_bytes!("../assets/art/templates-v1.png"),
    ),
    (
        "remap",
        "Transform keys as well as values",
        include_bytes!("../assets/art/remap-v1.png"),
    ),
    (
        "async",
        "Await work without losing its result",
        include_bytes!("../assets/art/async-v1.png"),
    ),
    (
        "exhaustive",
        "Account for every possible state",
        include_bytes!("../assets/art/exhaustive-v1.png"),
    ),
    (
        "api-state",
        "State determines the available data",
        include_bytes!("../assets/art/api-state-v1.png"),
    ),
    (
        "brands",
        "Give similar values distinct identities",
        include_bytes!("../assets/art/brands-v1.png"),
    ),
    (
        "validation",
        "Validate data at the boundary",
        include_bytes!("../assets/art/validation-v1.png"),
    ),
    (
        "repositories",
        "Separate storage from the domain",
        include_bytes!("../assets/art/repositories-v1.png"),
    ),
    (
        "events",
        "Each event carries the right payload",
        include_bytes!("../assets/art/events-v1.png"),
    ),
    (
        "capstone",
        "Compose contracts into one system",
        include_bytes!("../assets/art/capstone-v1.png"),
    ),
    (
        "project-tracker",
        "Keep task transitions predictable",
        include_bytes!("../assets/art/project-tracker-v1.png"),
    ),
    (
        "project-parser",
        "Parse first; report precise failures",
        include_bytes!("../assets/art/project-parser-v1.png"),
    ),
    (
        "project-catalog",
        "Preserve contracts across a catalog",
        include_bytes!("../assets/art/project-catalog-v1.png"),
    ),
];
pub fn caption(id: &str) -> &'static str {
    IMAGES
        .iter()
        .find(|(key, _, _)| *key == id)
        .map(|(_, caption, _)| *caption)
        .unwrap_or("Lesson illustration")
}
#[derive(Default)]
pub struct Illustrations {
    cache: VecDeque<(String, TextureHandle)>,
    pending: HashMap<String, mpsc::Receiver<Result<ColorImage, String>>>,
    errors: HashMap<String, String>,
}
impl Illustrations {
    pub fn get(&mut self, ctx: &egui::Context, id: &str) -> Option<TextureHandle> {
        // Drop stale results after navigation; a previous lesson can never replace this image.
        self.pending.retain(|key, _| key == id);
        if let Some(index) = self.cache.iter().position(|(key, _)| key == id)
            && let Some(entry) = self.cache.remove(index)
        {
            let texture = entry.1.clone();
            self.cache.push_front(entry);
            return Some(texture);
        }
        if let Some(receiver) = self.pending.get(id) {
            match receiver.try_recv() {
                Ok(Ok(image)) => {
                    self.pending.remove(id);
                    let texture = ctx.load_texture(
                        format!("lesson-art-{id}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.cache.push_front((id.into(), texture.clone()));
                    self.cache.truncate(4);
                    return Some(texture);
                }
                Ok(Err(error)) => {
                    self.pending.remove(id);
                    self.errors.insert(id.into(), error);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending.remove(id);
                    self.errors
                        .insert(id.into(), "Illustration could not load".into());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        } else if !self.errors.contains_key(id) {
            if let Some((_, _, bytes)) = IMAGES.iter().find(|(key, _, _)| *key == id) {
                let (tx, rx) = mpsc::channel();
                self.pending.insert(id.into(), rx);
                let bytes = *bytes;
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let image = image::load_from_memory(bytes)
                        .map_err(|e| e.to_string())
                        .map(|image| {
                            // Contain rather than crop: every educational object stays visible.
                            let rgba = image.thumbnail(1000, 480).to_rgba8();
                            ColorImage::from_rgba_unmultiplied(
                                [rgba.width() as usize, rgba.height() as usize],
                                rgba.as_raw(),
                            )
                        });
                    let _ = tx.send(image);
                    ctx.request_repaint();
                });
            } else {
                self.errors
                    .insert(id.into(), "No artwork for this lesson".into());
            }
        }
        if self.pending.contains_key(id) {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
        None
    }
    pub fn failed(&self, id: &str) -> bool {
        self.errors.contains_key(id)
    }
}

/// Validate shipped image delivery and identity coverage, independent of UI presentation.
pub fn verify(lessons: &[crate::course::Lesson]) -> Result<(), String> {
    let ids: std::collections::HashSet<_> = IMAGES.iter().map(|(id, _, _)| *id).collect();
    if ids.len() != IMAGES.len() || IMAGES.len() != lessons.len() {
        return Err("Every lesson needs one uniquely keyed illustration".into());
    }
    for lesson in lessons {
        let (_, _, bytes) = IMAGES
            .iter()
            .find(|(id, _, _)| *id == lesson.id)
            .ok_or_else(|| format!("Missing illustration for {}", lesson.id))?;
        let image = image::load_from_memory(bytes)
            .map_err(|e| format!("{}: {e}", lesson.id))?
            .to_rgba8();
        if image.width() < 900
            || image.height() < 400
            || !image.pixels().any(|p| p.0[3] == 0)
            || !image.pixels().any(|p| p.0[3] > 0)
        {
            return Err(format!(
                "{}: expected a visible high-resolution illustration with transparency",
                lesson.id
            ));
        }
    }
    println!(
        "Verified {} uniquely keyed, decodable transparent lesson illustrations.",
        IMAGES.len()
    );
    Ok(())
}
