use std::{collections::HashMap, hash::Hash};

use super::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructDefinition {
    pub name: String,
    fields: HashMap<String, Value>,
}

impl Hash for StructDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fields.values().for_each(|v| v.hash(state));
    }
}
