use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub section: String,
    pub title: String,
    pub concept: String,
    pub task: String,
    pub starter: String,
    pub solution: String,
    pub alternative: String,
    pub checks: String,
    pub hint: String,
    pub runtime: String,
    pub negative: String,
}

pub fn load() -> Result<Vec<Lesson>, String> {
    let lessons: Vec<Lesson> =
        serde_json::from_str(include_str!("../content/course.json")).map_err(|e| e.to_string())?;
    let mut ids = std::collections::HashSet::new();
    for lesson in &lessons {
        if lesson.id.is_empty()
            || !lesson
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c == b'-')
            || !ids.insert(&lesson.id)
            || [
                &lesson.title,
                &lesson.concept,
                &lesson.task,
                &lesson.starter,
                &lesson.solution,
                &lesson.alternative,
                &lesson.checks,
                &lesson.hint,
            ]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            return Err(format!("Invalid lesson: {}", lesson.id));
        }
    }
    if lessons.is_empty() {
        return Err("The course is empty".into());
    }
    Ok(lessons)
}

#[cfg(test)]
mod tests {
    #[test]
    fn course_has_six_sections_and_fifty_three_labs() {
        let lessons = super::load().unwrap();
        assert_eq!(lessons.len(), 53);
        assert_eq!(
            lessons
                .iter()
                .map(|l| &l.section)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            6
        );
        for l in lessons {
            assert_ne!(l.starter, l.solution, "{}", l.id);
        }
    }
}
