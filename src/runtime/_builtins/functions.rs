use {
    crate::runtime::{
        scope::Scope,
        types::{
            function::{BuiltinFunction, Function, FunctionOutline},
            ContextualValue, Value, ValueType,
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
                handler: Arc::new(Box::new(|_: &Scope, i: Vec<ContextualValue>| {
                    println!("{}", i[0].0);
                    None
                })),
            }
            .packaged(),
        ),
    );
}
