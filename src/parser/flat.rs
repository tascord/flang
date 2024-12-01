use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

pub static FLAT: LazyLock<Flat> = LazyLock::new(|| Flat::default());

#[derive(Default)]
pub struct Flat {
    source: RwLock<String>,
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
}
