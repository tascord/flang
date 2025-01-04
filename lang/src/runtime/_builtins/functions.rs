use {
    crate::runtime::{
        _builtins::traits::{_TraitToPretty, _TraitToString},
        scope::Scope,
        types::{
            function::{BuiltinFunction, Function, FunctionOutline},
            Value, ValueType,
        },
    },
    std::sync::Arc,
};

pub fn default_impl(s: &Scope) {
    s.declare(
        "print",
        Value::Function(
            BuiltinFunction {
                outline: FunctionOutline { inputs: vec![("value".to_string(), ValueType::Any)], returns: None },
                handler: Arc::new(Box::new(|s: &Scope| {
                    let v = (*s.get("value").unwrap()).clone();
                    let v = match s.get_trait_for((v.clone()).clone(), &_TraitToPretty.name) {
                        Some(pretty) => pretty
                            .get_function("to_pretty")
                            .unwrap()
                            .call(s, vec![v.clone().anonymous()])
                            .unwrap()
                            .unwrap()
                            .as_string()
                            .unwrap()
                            .to_string(),
                        None => match s.get_trait_for((v.clone()).clone(), &_TraitToString.name) {
                            Some(string) => string
                                .get_function("to_string")
                                .unwrap()
                                .call(s, vec![v.clone().anonymous()])
                                .unwrap()
                                .unwrap()
                                .as_string()
                                .unwrap()
                                .to_string(),
                            None => format!("[Debug: {}]", v),
                        },
                    };

                    println!("{}", v);
                    None
                })),
            }
            .packaged(),
        ),
    );
}
