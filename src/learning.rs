//! Authored teaching sequence, independent of the existing lab completion contracts.
use crate::course::Lesson;
use serde::Deserialize;
#[derive(Clone, Deserialize)]
pub struct Term {
    pub term: String,
    pub meaning: String,
}
#[derive(Clone, Deserialize)]
pub struct Prediction {
    pub question: String,
    pub choices: Vec<String>,
    pub correct: usize,
    pub explanation: String,
}
#[derive(Clone, Deserialize)]
pub struct Practice {
    pub task: String,
    pub starter: String,
    pub solution: String,
    pub alternative: String,
    pub negative: String,
    pub checks: String,
    pub runtime: String,
    pub hint: String,
    pub playground: String,
}
#[derive(Clone, Deserialize)]
pub struct Guide {
    pub id: String,
    pub outcome: String,
    pub prerequisites: Vec<String>,
    pub vocabulary: Vec<Term>,
    pub example: String,
    pub explanation: Vec<String>,
    pub prediction: Prediction,
    pub practice: Practice,
    pub transfer: Vec<String>,
}
impl Guide {
    pub fn lesson(&self, base: &Lesson) -> Lesson {
        let p = &self.practice;
        Lesson {
            id: format!("guided-{}", self.id),
            section: base.section.clone(),
            title: format!("Guided practice: {}", base.title),
            concept: self.explanation.join("\n\n"),
            task: p.task.clone(),
            starter: p.starter.clone(),
            solution: p.solution.clone(),
            alternative: p.alternative.clone(),
            negative: p.negative.clone(),
            checks: p.checks.clone(),
            runtime: p.runtime.clone(),
            hint: p.hint.clone(),
        }
    }
}
pub fn load(lessons: &[Lesson]) -> Result<Vec<Guide>, String> {
    let guides: Vec<Guide> = serde_json::from_str(include_str!("../content/learning.json"))
        .map_err(|e| e.to_string())?;
    if guides.len() != lessons.len() {
        return Err("Every lesson must have one structured guide".into());
    }
    let mut seen = std::collections::HashSet::new();
    for (guide, lesson) in guides.iter().zip(lessons) {
        if guide.id != lesson.id
            || guide.vocabulary.is_empty()
            || guide.explanation.len() < 2
            || guide.prediction.choices.len() != 3
            || guide.prediction.correct >= 3
            || guide.prerequisites.iter().any(|id| !seen.contains(id))
            || [
                guide.outcome.as_str(),
                guide.example.as_str(),
                guide.prediction.question.as_str(),
                guide.prediction.explanation.as_str(),
                guide.practice.playground.as_str(),
            ]
            .iter()
            .any(|s| s.trim().is_empty())
            || guide
                .vocabulary
                .iter()
                .any(|term| term.term.is_empty() || term.meaning.is_empty())
        {
            return Err(format!("Invalid learning guide: {}", guide.id));
        }
        seen.insert(guide.id.clone());
    }
    // Reuse domain validation for all of the independently graded practice contracts.
    for guide in &guides {
        let p = &guide.practice;
        if [
            &p.task,
            &p.starter,
            &p.solution,
            &p.alternative,
            &p.negative,
            &p.checks,
            &p.hint,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || p.starter == p.solution
        {
            return Err(format!("Invalid practice contract: {}", guide.id));
        }
    }
    Ok(guides)
}
/// Validate teaching examples and real hands-on contracts independently from course completion.
pub fn verify(lessons: &[Lesson], guides: &[Guide]) -> Result<(), String> {
    let practices: Vec<_> = guides
        .iter()
        .zip(lessons)
        .map(|(guide, lesson)| guide.lesson(lesson))
        .collect();
    crate::compiler::verify(&practices)?;
    for ((guide, base), practice) in guides.iter().zip(lessons).zip(&practices) {
        let mut example = base.clone();
        example.checks = String::new();
        example.runtime = String::new();
        let report = crate::compiler::check(&example, &format!("{}\nexport {{}};", guide.example))?;
        if !report.passed {
            return Err(format!("{} / worked example: {}", guide.id, report.text));
        }
        let session = crate::engine::Session::start(crate::engine::Request {
            debug: false,
            breakpoints: Default::default(),
            answer: practice.solution.clone(),
            playground: guide.practice.playground.clone(),
        });
        loop {
            match session
                .events
                .recv_timeout(std::time::Duration::from_secs(30))
                .map_err(|e| e.to_string())?
            {
                crate::engine::Event::Finished { success: true } => break,
                crate::engine::Event::Problems { text }
                | crate::engine::Event::Failure { text } => {
                    return Err(format!("{} / practice playground: {text}", guide.id));
                }
                crate::engine::Event::Finished { success: false } => {
                    return Err(format!("{} / practice playground failed", guide.id));
                }
                _ => {}
            }
        }
        println!(
            "{}: explained example compiles; guided playground runs",
            guide.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_lesson_has_prior_only_prerequisites_and_a_separate_practice() {
        let lessons = crate::course::load().unwrap();
        let guides = super::load(&lessons).unwrap();
        assert_eq!(guides.len(), 53);
        for (guide, lesson) in guides.iter().zip(&lessons) {
            let practice = guide.lesson(lesson);
            assert_ne!(practice.id, lesson.id);
            assert_ne!(practice.checks, lesson.checks);
        }
    }
}
