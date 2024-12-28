use {
    super::{
        traits::{TraitDefinition, TraitInstance},
        types::{structs::StructDefinition, Value, ValueType},
    },
    anyhow::{anyhow, bail},
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

#[derive(Default, Debug)]
pub struct Scope {
    traits: RwLock<HashMap<Arc<TraitDefinition>, Arc<RwLock<Vec<TraitInstance>>>>>,
    variables: RwLock<HashMap<String, Arc<Value>>>,
    structs: RwLock<HashMap<String, Arc<StructDefinition>>>,
    for_var: Option<Arc<Value>>,
    export: RwLock<Option<Arc<Scope>>>,
}

impl Scope {
    pub fn new() -> Self { Scope::default() }

    pub fn child_for_var(&self, v: Value) -> Self {
        let mut c = Scope::new();
        c.for_var = Some(Arc::new(v.clone()));

        // If a struct, add all fields to the scope
        if let Some((_, fields)) = v.as_struct_instance() {
            fields.iter().for_each(|v| c.declare(&v.0, v.1.clone()));
        }

        // Get functions from traits
        for t in self.get_traits_for(v) {
            t.def.functions.iter().chain(t.overrides.iter()).for_each(|f| {
                c.declare(&f.0, Value::Function(t.get_function(&f.0).unwrap()));
            })
        }

        // Copy over in-scope traits
        c.traits = RwLock::new(self.traits.read().unwrap().clone());

        c
    }

    pub fn container(&self) -> Option<Arc<Value>> { self.for_var.clone() }

    pub fn child(&self) -> Self {
        let mut c = Scope::new();
        c.variables = RwLock::new(self.variables.read().unwrap().clone());
        c.traits = RwLock::new(self.traits.read().unwrap().clone());
        c.structs = RwLock::new(self.structs.read().unwrap().clone());
        c
    }

    pub fn get_traits_for(&self, v: Value) -> Vec<TraitInstance> {
        self.traits
            .read()
            .unwrap()
            .iter()
            .filter(|(def, _)| def.restriction.clone().map(|r| r.matches(&v, self)).unwrap_or(true))
            .flat_map(|(_, i)| {
                i.read().unwrap().clone().into_iter().filter(|i| i.restriction.matches(&v, self)).collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn get_trait_for(&self, v: Value, n: &str) -> Option<TraitInstance> {
        self.get_traits_for(v).into_iter().find(|t| t.def.name == n.to_string())
    }

    pub fn define_struct(&self, name: &str, def: StructDefinition) {
        self.export.read().unwrap().clone().inspect(|e| e.define_struct(name, def.clone()));
        self.structs.write().unwrap().insert(name.to_string(), Arc::new(def));
    }

    pub fn get_structdef(&self, name: &str) -> Option<Arc<StructDefinition>> {
        self.structs.read().unwrap().get(name).cloned()
    }

    pub fn implements(&self, v: &Value, t: &TraitDefinition) -> bool {
        self.traits
            .read()
            .unwrap()
            .get(t)
            .map(|t| t.read().unwrap().clone().into_iter().find(|i| i.matches(v, &self)))
            .is_some()
    }

    pub fn declare_trait(&self, t: &TraitDefinition) {
        self.export.read().unwrap().clone().inspect(|e| e.declare_trait(t));
        self.traits.write().unwrap().insert(t.clone().into(), Default::default());
    }

    pub fn implement_trait(&self, n: &str, f: impl Fn(Arc<TraitDefinition>) -> TraitInstance + Clone) -> anyhow::Result<()> {
        self.export.read().unwrap().clone().inspect(|e| e.implement_trait(n, f.clone()).unwrap());
        let binding = (*self.traits.read().unwrap()).clone();
        let def = binding
            .keys()
            .find(|k| k.name == n.to_string())
            .ok_or(anyhow!("No trait named {} available to implement.", n))?;

        let t = f(def.clone());
        let mut binding = self.traits.write().unwrap();
        binding.get_mut(def).unwrap().write().unwrap().push(t.clone());

        Ok(())
    }

    pub fn get_trait(&self, t: &str) -> Option<(Arc<TraitDefinition>, Arc<RwLock<Vec<TraitInstance>>>)> {
        self.traits.read().unwrap().iter().find(|(d, _)| d.name == t.to_string()).map(|v| (v.0.clone(), v.1.clone()))
    }

    pub fn declare(&self, var: &str, value: Value) {
        self.export.read().unwrap().clone().inspect(|e| e.declare(var, value.clone()));
        self.variables.write().unwrap().insert(var.to_string(), value.into());
    }

    pub fn assign(&self, var: &str, value: Value) -> anyhow::Result<()> {
        let mut binding = self.variables.write().unwrap();
        let ex = binding.get_mut(var).ok_or(anyhow!("No variable named {} to re-assign to.", var))?;

        let ex_t = <Value as Into<ValueType>>::into(<Value as Clone>::clone(&*ex.clone())).clone();
        // let v_t = <Value as Into<ValueType>>::into(value.clone());

        if !ex_t.matches(&value, self) {
            // TODO: impl display on valuetype
            bail!("Can't assign value of type ? to variable {}, which has type ?", var);
        }

        *ex = value.into();

        Ok(())
    }

    pub fn get(&self, var: &str) -> Option<Arc<Value>> { self.variables.read().unwrap().get(var).cloned() }

    pub fn use_export(&self, s: Arc<Scope>) { self.export.write().unwrap().replace(s); }

    pub fn clear_export(&self) { self.export.write().unwrap().take(); }

    pub fn absorb(&self, s: Arc<Scope>) {
        self.structs.write().unwrap().extend(s.structs.read().unwrap().clone().into_iter());
        self.variables.write().unwrap().extend(s.variables.read().unwrap().clone().into_iter());
        self.traits.write().unwrap().extend(s.traits.read().unwrap().clone().into_iter());
    }
}
