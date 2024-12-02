use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

use anyhow::bail;

use crate::runtime::scope::Scope;

use super::parse;

#[derive(Default)]
pub struct FlatManager {
    exports: HashMap<String, Arc<Scope>>,
}

pub static FLAT_INSTANCE: LazyLock<FlatManager> = LazyLock::new(|| FlatManager::default());

#[derive(Default)]
pub struct Flat {
    source: RwLock<String>,
    exports: Arc<Scope>,
    file_bounds: RwLock<HashMap<String, (usize, usize)>>,
}

impl Flat {
    pub fn add_source_file(&self, contents: String, relative_path: String) {
        let mut source = self.source.write().unwrap();

        let header = format!("\n\n// {relative_path}\n");
        let bound = source.len();

        (*source).push_str(&format!("{}{}", header, contents));
        self.file_bounds.write().unwrap().insert(relative_path, (bound, bound + contents.len()));
    }

    pub fn get_source_file(&self, relative_path: String) -> Option<(String, usize)> {
        if let Some(bound) = self.file_bounds.read().unwrap().get(&relative_path) {
            let slice = &self.source.read().unwrap()[bound.0..bound.1];
            Some((slice.to_string(), bound.0))
        } else {
            None
        }
    }

    pub fn process(p: PathBuf) -> anyhow::Result<()> {
        if p.is_dir() {
            if p.join("main.fl").exists() {
                return Self::process(p.join("main.fl"));
            }
        }

        if !p.exists() {
            bail!("Source file {} does not exist", p.display());
        }

        let mut buf = String::new();
        File::open(p).unwrap().read_to_string(&mut buf)?;
        let tree = parse(buf.leak())?;

        Ok(())
    }
}
