use {
    super::{
        scope::Scope,
        types::{
            function::{Function, FunctionOutline},
            Value, ValueType,
        },
    },
    std::{collections::HashMap, hash::Hash, sync::Arc},
};

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub outlines: HashMap<String, FunctionOutline>,
    pub functions: HashMap<String, Arc<Box<dyn Function>>>,
    pub restriction: Option<Box<ValueType>>,
}

impl PartialEq for TraitDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.outlines == other.outlines
            && self.functions.values().zip(other.functions.values()).all(|(a, b)| Arc::ptr_eq(a, b))
            && self.restriction == other.restriction
    }
}

impl Hash for TraitDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.outlines.values().for_each(|v| v.hash(state));
        self.functions.values().for_each(|v| Arc::as_ptr(v).hash(state));
        self.restriction.hash(state);
    }
}

impl Eq for TraitDefinition {}

#[derive(Clone)]
pub struct TraitInstance {
    pub def: Arc<TraitDefinition>,
    pub overrides: HashMap<String, Arc<Box<dyn Function>>>,
    pub restriction: Box<ValueType>,
}

impl TraitInstance {
    pub fn get_function(&self, name: &str) -> Option<Arc<Box<dyn Function>>> {
        self.overrides.get(name).or(self.def.functions.get(name)).cloned()
    }

    pub fn matches(&self, v: &Value, s: &Scope) -> bool {
        self.def.restriction.clone().map(|r| r.matches(v, s)).unwrap_or(true) && self.restriction.matches(v, s)
    }
}
