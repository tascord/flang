use {
    crate::runtime::{
        scope::Scope,
        traits::{TraitDefinition, TraitInstance},
        types::{
            function::{BuiltinFunction, Function, FunctionOutline},
            Value, ValueType,
        },
    },
    owo_colors::OwoColorize,
    std::sync::{Arc, LazyLock},
};

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

#[allow(non_upper_case_globals)]
pub static _TraitToPretty: LazyLock<TraitDefinition> = LazyLock::new(|| TraitDefinition {
    name: "ToPretty".to_string(),
    outlines: map! {
        "to_pretty".to_string() => FunctionOutline {
            inputs: vec![ ("self".to_string(), ValueType::This)],
            returns: Some(ValueType::String),
        }
    },
    functions: map! {},
    restriction: None,
});

pub fn default_impl(s: &Scope) {
    s.declare_trait(&_TraitToString);
    s.implement_trait(&_TraitToString.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::Any),
        overrides: map! {
            "to_string".to_string() => BuiltinFunction {
                outline: _TraitToString.outlines.get("to_string").unwrap().clone(),
                handler: Arc::new(Box::new(|s: &Scope| {
                    let ret = Value::String(s.get("self").unwrap().to_string()).anonymous();
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
                handler: Arc::new(Box::new(|s: &Scope| {
                    let ret = Value::String(format!("{}{}", s.get("left").unwrap().as_string().unwrap(), s.get("right").unwrap().as_string().unwrap())).anonymous();
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
                handler: Arc::new(Box::new(|s: &Scope| {
                    let ret = Value::Number(s.get("left").unwrap().as_number().unwrap() + s.get("right").unwrap().as_number().unwrap()).anonymous();
                    Some(ret)
                })),
            }.packaged()
        },
    })
    .unwrap();

    s.declare_trait(&_TraitToPretty);
    s.implement_trait(&_TraitToPretty.name, |def| TraitInstance {
        def,
        restriction: Box::new(ValueType::Any),
        overrides: map! {
            "to_pretty".to_string() => BuiltinFunction {
                outline: _TraitToPretty.outlines.get("to_pretty").unwrap().clone(),
                handler: Arc::new(Box::new(|s: &Scope| {
                    let v = match &*s.get("self").unwrap() {
                        Value::Number(v) => v.to_string().yellow().to_string(),
                        Value::Boolean(v) => v.to_string().green().to_string(),
                        Value::String(v) => format!("\"{v}\"").cyan().to_string(),
                        Value::StructInstance(struct_definition, hash_map) => format!(
                            "{name} {left} {body} {right}",
                            name = struct_definition.name,
                            left = "{".blue().to_string(), right = "}".blue().to_string(),
                            body = hash_map.into_iter().map(|(k, v)| {
                                let binding = s.get_trait_for(v.clone(), "ToPretty").unwrap().get_function("to_pretty").unwrap().call(s, vec![v.clone().anonymous()]).unwrap().unwrap();
                                let v = binding.as_string().unwrap();
                                format!("{k}: {v}")
                            }).collect::<Vec<_>>().join(", ")
                        ),
                        Value::Function(arc) => format!("{:?}", (*arc).clone()).magenta().to_string(),
                        Value::Undefined => "null".dimmed().to_string(),
                    };

                    Some(Value::String(v).anonymous())
                })),
            }.packaged()
        },
    })
    .unwrap();
}
