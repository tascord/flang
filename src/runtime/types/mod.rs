use {
    super::{scope::Scope, traits::TraitDefinition},
    crate::project::source::LinkedSpan,
    enum_as_inner::EnumAsInner,
    function::{Function, FunctionOutline},
    pest::Span,
    std::{
        collections::HashMap,
        fmt::{Debug, Display},
        hash::Hash,
        ops::Deref,
        sync::Arc,
    },
    structs::StructDefinition,
};

pub mod function;
pub mod structs;

#[derive(Clone, Debug)]
pub struct ContextualValue(pub Value, pub LinkedSpan);
impl Deref for ContextualValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Clone, EnumAsInner, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    StructInstance(StructDefinition, HashMap<String, Value>),
    Function(Arc<Box<dyn Function>>),
    Undefined,
    External(String, Arc<Scope>),
    Return(Box<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::StructInstance(struct_definition, hash_map) => write!(f, "{} {:?}", struct_definition.name, hash_map),
            Value::Function(arc) => write!(f, "{:?}", *arc),
            Value::Undefined => write!(f, "[Undefined]"),
            Value::External(name, ..) => write!(f, "[Export {name}]"),
            Value::Return(value) => std::fmt::Display::fmt(&*value, f),
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Number(v) => v.to_string().hash(state),
            Value::String(v) => v.hash(state),
            Value::Boolean(v) => v.hash(state),
            Value::StructInstance(struct_definition, hash_map) => {
                struct_definition.hash(state);
                hash_map.values().for_each(|v| v.hash(state));
            }
            Value::Function(arc) => Arc::as_ptr(arc).hash(state),
            Value::Undefined => {}
            Value::External(pkg, ..) => pkg.hash(state),
            Value::Return(value) => std::hash::Hash::hash(&*value, state),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::StructInstance(l0, l1), Self::StructInstance(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Function(l0), Self::Function(r0)) => Arc::ptr_eq(l0, r0),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for Value {}

impl From<f64> for Value {
    fn from(value: f64) -> Self { Self::Number(value) }
}

impl From<String> for Value {
    fn from(value: String) -> Self { Self::String(value) }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self { Self::Boolean(value) }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self { value.map(|v| v.into()).unwrap_or(Value::Undefined) }
}

impl Into<ValueType> for Value {
    fn into(self) -> ValueType {
        match self {
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
            Value::StructInstance(def, ..) => ValueType::StructInstance(def),
            Value::Function(fun) => ValueType::Function(Box::new(fun.outline())),
            Value::Undefined => ValueType::Undefined,
            Value::External(name, ..) => ValueType::Export(name),
            Value::Return(value) => Into::<ValueType>::into(*value),
        }
    }
}

impl Value {
    pub fn context(self, s: LinkedSpan) -> ContextualValue { ContextualValue(self, s) }

    pub fn anonymous(self) -> ContextualValue {
        ContextualValue(self, LinkedSpan(Span::new("", 0, 0).unwrap(), String::new()))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Number,
    String,
    Boolean,
    StructInstance(StructDefinition),
    Function(Box<FunctionOutline>),
    Undefined,
    This,
    Any,
    Implements(TraitDefinition),
    Export(String),
}

impl Debug for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number => write!(f, "Number"),
            Self::String => write!(f, "String"),
            Self::Boolean => write!(f, "Boolean"),
            Self::StructInstance(_) => write!(f, "StructInstance"),
            Self::Function(_) => write!(f, "Function"),
            Self::Undefined => write!(f, "Undefined"),
            Self::This => write!(f, "Self"),
            Self::Any => write!(f, "Any"),
            Self::Implements(_) => write!(f, "Implements"),
            Self::Export(v) => write!(f, "Export({v})"),
        }
    }
}

impl ValueType {
    pub fn matches(&self, v: &Value, s: &Scope) -> bool {
        match self {
            ValueType::Any => true,
            ValueType::This => {
                s.container().map(|v| <Value as Into<ValueType>>::into((*v).clone()).matches(&v, s)).unwrap_or(false)
            }
            ValueType::Implements(def) => s.implements(v, def),
            t => <Value as Into<ValueType>>::into(v.clone()) == t.clone(),
        }
    }

    pub fn from_str(t: &str, s: &Scope) -> Option<ValueType> {
        match t {
            "number" => Some(ValueType::Number),
            "string" => Some(ValueType::String),
            "bool" => Some(ValueType::Boolean),
            "null" => Some(ValueType::Undefined),
            "any" => Some(ValueType::Any),

            v if v.starts_with("uses ") => {
                s.get_trait(v.strip_prefix("uses ").unwrap()).map(|v| ValueType::Implements((*v.0).clone()))
            }

            v => s.get_structdef(v).map(|v| ValueType::StructInstance((*v).clone())), // TODO: Function
        }
    }
}

impl Into<LinkedSpan> for ContextualValue {
    fn into(self) -> LinkedSpan { self.1 }
}
