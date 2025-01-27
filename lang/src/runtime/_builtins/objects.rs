use std::{
    collections::HashMap,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    function,
    runtime::{
        _builtins::traits::{_TraitToPretty, _TraitToString},
        scope::Scope,
        types::{structs::StructDefinition, Value, ValueType},
    },
};

#[macro_export]
macro_rules! struct_def {
    (name: ident, {$( $arg: ident: $type: expr ),*}) => {
        StructDefinition {
            name: stringify!($name),
            fields: {
                let mut map = HashMap::new();
                $(map.insert(stringify!($arg).to_string(), $type);)*
                map
            }
        }
    };
}

#[macro_export]
macro_rules! struct_inst {
    ($definition: expr, {$( $arg: ident: $value: expr ),*}) => {
        Value::StructInstance (
            $definition,
            {
                let mut map = HashMap::new();
                $(map.insert(stringify!($arg).to_string(), $value);)*
                map
            }
        )
    };
}

macro_rules! builtin_struct {
    ($scope: expr, $name: ident, {$( $arg: ident: $value: expr ),*}) => {
        let definition = StructDefinition {
            name: stringify!($name).to_string(),
            fields: {
                let mut map = HashMap::new();
                $(map.insert(stringify!($arg).to_string(), Into::<ValueType>::into($value.clone()));)*
                map
            }
        };

        $scope.define_struct(stringify!($name), definition.clone());
        $scope.declare(stringify!($name), Value::StructInstance (
            definition,
            {
                let mut map = HashMap::new();
                $(map.insert(stringify!($arg).to_string(), $value);)*
                map
            }
        ));
    };
}

pub fn default_impl(s: &Scope) {
    builtin_struct!(s, term, {
        println: function!((value: ValueType::Any) => None, |scope: &Scope| {
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
        })
    });

    builtin_struct!(s, time, {
        current_unix: function!(() => Some(ValueType::Number), |_: &Scope| {
            Some(Value::Number(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64).anonymous())
        })
    });
}
