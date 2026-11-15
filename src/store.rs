use crate::course::Lesson;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
pub struct Draft {
    pub source: String,
    #[serde(default)]
    pub playground: Option<String>,
    #[serde(default)]
    pub breakpoints: crate::engine::Breakpoints,
    pub passed_source: Option<String>,
    pub solution_seen: bool,
}
impl Draft {
    pub fn passed(&self) -> bool {
        self.passed_source.as_ref() == Some(&self.source)
    }
}
#[derive(Serialize, Deserialize)]
pub struct Progress {
    pub version: u32,
    pub selected: String,
    pub drafts: BTreeMap<String, Draft>,
    #[serde(default)]
    pub guided: BTreeMap<String, Draft>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            version: 1,
            selected: String::new(),
            drafts: BTreeMap::new(),
            guided: BTreeMap::new(),
        }
    }
}
impl Progress {
    pub fn draft(&mut self, lesson: &Lesson) -> &mut Draft {
        self.drafts
            .entry(lesson.id.clone())
            .or_insert_with(|| Draft {
                source: lesson.starter.clone(),
                ..Default::default()
            })
    }
    pub fn guided_draft(&mut self, lesson: &Lesson, playground: &str) -> &mut Draft {
        self.guided
            .entry(lesson.id.clone())
            .or_insert_with(|| Draft {
                source: lesson.starter.clone(),
                playground: Some(playground.into()),
                ..Default::default()
            })
    }
    pub fn complete(&self) -> usize {
        self.drafts.values().filter(|d| d.passed()).count()
    }
}

pub fn home() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("TYPEDOJO_HOME") {
        return Ok(path.into());
    }
    let user = std::env::var_os("HOME").ok_or("HOME is unavailable; set TYPEDOJO_HOME")?;
    Ok(PathBuf::from(user).join("Library/Application Support/TypeDojo"))
}

pub struct Store {
    pub root: PathBuf,
    _lock: fs::File,
}
impl Store {
    pub fn open(root: PathBuf) -> Result<(Self, Progress), String> {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let lock_path = root.join("session.lock");
        let lock = fs::OpenOptions::new().write(true).create_new(true).open(&lock_path)
            .map_err(|e| format!("Cannot open {}: {e}. Close other TypeDojo windows. If a previous session crashed, remove only this lock file and reopen.", lock_path.display()))?;
        let store = Self { root, _lock: lock };
        let progress = load(&store.root.join("progress.json"))?;
        Ok((store, progress))
    }
    pub fn save(&self, progress: &Progress) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(progress).map_err(|e| e.to_string())?;
        let tmp = self.root.join("progress.pending");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| e.to_string())?;
        fs::rename(tmp, self.root.join("progress.json")).map_err(|e| e.to_string())
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.root.join("session.lock"));
    }
}
fn load(path: &Path) -> Result<Progress, String> {
    match fs::read(path) {
        Ok(bytes) => {
            let p: Progress = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Progress was preserved, but could not be read: {e}"))?;
            if p.version != 1 {
                return Err(format!(
                    "Unsupported progress version {}. File preserved.",
                    p.version
                ));
            }
            Ok(p)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Progress::default()),
        Err(e) => Err(format!("Cannot read progress; file preserved: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "typedojo-store-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }
    #[test]
    fn progress_survives_reopen_and_edits_invalidate_pass() {
        let root = root();
        let (store, mut p) = Store::open(root.clone()).unwrap();
        let l = crate::course::load().unwrap().remove(0);
        let d = p.draft(&l);
        d.source = l.solution.clone();
        d.passed_source = Some(d.source.clone());
        d.playground = Some("console.log('saved playground');".into());
        d.breakpoints.insert("answer.ts".into(), [2, 4].into());
        assert!(d.passed());
        store.save(&p).unwrap();
        assert!(Store::open(root.clone()).is_err());
        drop(store);
        let (store, mut p) = Store::open(root.clone()).unwrap();
        assert_eq!(p.complete(), 1);
        assert_eq!(
            p.draft(&l).playground.as_deref(),
            Some("console.log('saved playground');")
        );
        assert_eq!(
            p.draft(&l).breakpoints.get("answer.ts"),
            Some(&[2, 4].into())
        );
        p.draft(&l).source.push(' ');
        assert_eq!(p.complete(), 0);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn corrupt_and_newer_progress_are_preserved() {
        let root = root();
        fs::create_dir_all(&root).unwrap();
        for text in ["broken", r#"{"version":2,"selected":"","drafts":{}}"#] {
            fs::write(root.join("progress.json"), text).unwrap();
            assert!(Store::open(root.clone()).is_err());
            assert_eq!(
                fs::read_to_string(root.join("progress.json")).unwrap(),
                text
            );
            assert!(!root.join("session.lock").exists());
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn old_drafts_load_without_workspace_fields() {
        let old = r#"{"version":1,"selected":"values","drafts":{"values":{"source":"export const n=1;","passed_source":"export const n=1;","solution_seen":false}}}"#;
        let progress: Progress = serde_json::from_str(old).unwrap();
        let draft = &progress.drafts["values"];
        assert!(draft.passed());
        assert!(draft.playground.is_none());
        assert!(draft.breakpoints.is_empty());
        assert!(progress.guided.is_empty());
    }
    #[test]
    fn guided_completion_is_separate_and_survives_reopen() {
        let root = root();
        let (store, mut progress) = Store::open(root.clone()).unwrap();
        let lesson = crate::course::load().unwrap().remove(0);
        let guide = crate::learning::load(&crate::course::load().unwrap())
            .unwrap()
            .remove(0);
        let practice = guide.lesson(&lesson);
        let draft = progress.guided_draft(&practice, &guide.practice.playground);
        draft.source = practice.solution;
        draft.passed_source = Some(draft.source.clone());
        assert!(draft.passed());
        assert_eq!(progress.complete(), 0);
        store.save(&progress).unwrap();
        drop(store);
        let (store, progress) = Store::open(root.clone()).unwrap();
        assert!(progress.guided[&practice.id].passed());
        assert_eq!(progress.complete(), 0);
        assert!(progress.drafts.is_empty());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
