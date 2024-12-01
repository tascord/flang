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

#[allow(non_upper_case_globals)]
pub static _Add: LazyLock<TraitDefinition> = LazyLock::new(|| TraitDefinition {
    name: "Add".to_string(),
    outlines: map! {
        "add".to_string() => FunctionOutline {
            inputs: vec![ ("left".to_string(), ValueType::This), ("right".to_string(), ValueType::This)],
            returns: Some(ValueType::Any),
        }
    },
    functions: map! {},
    restriction: None,
});

#[allow(non_upper_case_globals)]
pub static _TraitToString: LazyLock<TraitDefinition> = LazyLock::new(|| TraitDefinition {
    name: "ToString".to_string(),
    outlines: map! {
        "to_string".to_string() => FunctionOutline {
            inputs: vec![ ("self".to_string(), ValueType::This)],
            returns: Some(ValueType::String),
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

    s.declare_trait(&_TraitToString);
    s.implement_trait(&_TraitToString.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::Any),
        overrides: map! {
            "to_string".to_string() => BuiltinFunction {
                outline: _TraitIndexable.outlines.get("index").unwrap().clone(),
                handler: Arc::new(Box::new(|_: &Scope, i: Vec<ContextualValue>| {
                    let ret = Value::String(i[0].to_string()).anonymous();
                    Some(ret)
                })),
            }.packaged()
        },
    })
    .unwrap();

    s.declare_trait(&_Add);
    s.implement_trait(&_Add.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::String),
        overrides: map! {
            "add".to_string() => BuiltinFunction {
                outline: _Add.outlines.get("add").unwrap().clone(),
                handler: Arc::new(Box::new(|_: &Scope, i: Vec<ContextualValue>| {
                    let ret = Value::String(format!("{}{}", i[0].as_string().unwrap(), i[1].as_string().unwrap())).anonymous();
                    Some(ret)
                })),
            }.packaged()
        },
    })
    .unwrap();
    s.implement_trait(&_Add.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::Number),
        overrides: map! {
            "add".to_string() => BuiltinFunction {
                outline: _Add.outlines.get("add").unwrap().clone(),
                handler: Arc::new(Box::new(|_: &Scope, i: Vec<ContextualValue>| {
                    let ret = Value::Number(i[0].as_number().unwrap() + i[1].as_number().unwrap()).anonymous();
                    Some(ret)
                })),
            }.packaged()
        },
    })
    .unwrap();
}
