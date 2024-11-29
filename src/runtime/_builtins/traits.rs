use {
    crate::runtime::{
        scope::Scope,
        traits::{TraitDefinition, TraitInstance},
        types::{
            function::{BuiltinFunction, Function, FunctionOutline},
            ContextualValue, Value, ValueType,
        },
    },
    std::sync::{Arc, LazyLock},
};

#[allow(non_upper_case_globals)]
pub static _TraitIndexable: LazyLock<TraitDefinition> = LazyLock::new(|| TraitDefinition {
    name: "Indexable".to_string(),
    outlines: map! {
        "index".to_string() => FunctionOutline {
            inputs: vec![ ("self".to_string(), ValueType::This), ("idx".to_string(), ValueType::Any)],
            returns: Some(ValueType::Any),
        }
    },
    functions: map! {},
    restriction: None,
});

pub fn default_impl(s: &Scope) {
    s.declare_trait(&_TraitIndexable);
    s.implement_trait(&_TraitIndexable.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::String),
        overrides: map! {
            "index".to_string() => BuiltinFunction {
                outline: _TraitIndexable.outlines.get("index").unwrap().clone(),
                handler: Arc::new(Box::new(|_: &Scope, i: Vec<ContextualValue>| {
                    let (v, i) = (i[0].as_string().unwrap(), i[1].as_number().unwrap());
                     Some(Value::from(v.chars().skip((*i as usize).saturating_sub(1)).map(|c| c.to_string()).next()).anonymous())
                })),
            }.packaged()
        },
    })
    .unwrap();
}
