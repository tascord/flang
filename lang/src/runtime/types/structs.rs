use std::{collections::HashMap, hash::Hash};

use super::ValueType;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructDefinition {
    pub name: String,
    pub fields: HashMap<String, ValueType>,
}

impl Hash for StructDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fields.values().for_each(|v| v.hash(state));
    }
}
