use {
    pest::{iterators::Pair, RuleType, Span},
    std::{
        collections::BTreeMap, fmt::Debug, path::Path, sync::{Arc, LazyLock, RwLock}
    },
};

pub static SOURCES: LazyLock<SourceMap> = LazyLock::new(|| SourceMap(Default::default()));

pub struct SourceMap(RwLock<BTreeMap<String, Arc<String>>>);

#[derive(Clone)]
pub struct LinkedSpan(pub Span<'static>, pub String);

impl Debug for LinkedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LinkedSpan").field(&self.1).finish()
    }
}

impl LinkedSpan {
    pub fn span(&self) -> Span<'static> { self.0.clone() }

    pub fn file_nameish(&self) -> String {
        let mut folder = Path::new(&self.1).parent().unwrap();
        let mut depth = 0;

        loop {
            if let Some(_) =
                folder.read_dir().unwrap().find(|f| f.as_ref().map(|v| v.file_name() == "manifest.json").unwrap_or_default())
            {
                return Path::new(&self.1)
                    .display()
                    .to_string()
                    .replace(&folder.parent().unwrap().display().to_string(), "")
                    .split_at(1)
                    .1
                    .to_string();
            }

            if let Some(parent) = folder.parent() {
                folder = parent
            } else {
                break;
            }

            depth = depth + 1;
        }

        self.1.clone()
    }

    pub fn file_name(&self) -> String { self.1.to_string() }

    pub fn file_contents(&self) -> Arc<String> { SOURCES.get_source(self.1.clone()).unwrap() }
}

impl SourceMap {
    pub fn add_source(&self, p: String, s: String) -> Arc<String> {
        let a = Arc::new(s);
        self.0.write().unwrap().insert(p, a.clone());
        a
    }

    pub fn get_source(&self, p: String) -> Option<Arc<String>> { self.0.read().unwrap().get(&p).cloned() }
}

pub trait Saucy {
    fn source(self, p: String) -> LinkedSpan;
}

impl<R> Saucy for Pair<'static, R>
where
    R: RuleType,
{
    fn source(self, p: String) -> LinkedSpan { LinkedSpan(self.as_span(), p) }
}
