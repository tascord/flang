use crate::runtime::{
    _builtins::traits::{_TraitToPretty, _TraitToString},
    scope::Scope,
    types::ValueType,
};

#[macro_export]
macro_rules! function {
    (($( $arg: ident: $type: expr ),*) => $returns: expr, $func: expr) => {
        {
            use crate::runtime::types::function::Function;
            crate::runtime::types::Value::Function(
                    crate::runtime::types::function::BuiltinFunction {
                        outline: crate::runtime::types::function::FunctionOutline {
                            inputs: vec![ $((stringify!($arg).to_string(), $type),)* ],
                            returns: $returns
                        },
                        handler: std::sync::Arc::new(Box::new(|scope: &Scope| {
                            $func(scope)
                        }))
                    }.packaged()
                )
            }
    };
}

pub fn default_impl(s: &Scope) {
    s.declare(
        "print",
        function!((value: ValueType::Any) => None, |scope: &Scope| {
            let value = (*scope.get("value").unwrap()).clone();
            let format = match scope.get_trait_for(value.clone(), &_TraitToPretty.name) {
                Some(pretty) => pretty
                    .get_function("to_pretty")
                    .unwrap()
                    .call(scope, vec![value.clone().anonymous()])
                    .unwrap()
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .to_string(),
                None => match scope.get_trait_for((value.clone()).clone(), &_TraitToString.name) {
                    Some(string) => string
                        .get_function("to_string")
                        .unwrap()
                        .call(scope, vec![value.clone().anonymous()])
                        .unwrap()
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .to_string(),
                    None => format!("[Debug: {}]", value),
                },

            };

            println!("{}", format);
            None
        }),
    );
}
