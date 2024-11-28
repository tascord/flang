use {
    super::{Value, ValueType},
    crate::runtime::scope::Scope,
    std::{fmt::Debug, sync::Arc},
};

pub trait Function: Sync + Send + Debug {
    fn call(&self, scope: &Scope, inputs: Vec<Value>) -> Option<Value>;
    fn outline(&self) -> FunctionOutline;
    fn packaged(self) -> Arc<Box<dyn Function>>;
}

#[derive(Clone)]
pub struct BasicFunction {
    pub outline: FunctionOutline,
}

#[derive(Clone)]
pub struct BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<Value>) -> Option<Value> + Sync + Send + Clone + 'static,
{
    pub outline: FunctionOutline,
    pub handler: Arc<Box<T>>,
}

impl<T> Debug for BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<Value>) -> Option<Value> + Sync + Send + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "[Builtin Function]") }
}

impl<T> Function for BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<Value>) -> Option<Value> + Sync + Send + Clone + 'static,
{
    fn call(&self, scope: &Scope, inputs: Vec<Value>) -> Option<Value> { (self.handler.clone())(scope, inputs) }

    fn outline(&self) -> FunctionOutline { todo!() }

    fn packaged(self) -> Arc<Box<dyn Function>> { Arc::new(Box::new(self) as Box<dyn Function>) }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FunctionOutline {
    pub inputs: Vec<ValueType>,
    pub returns: Option<ValueType>,
}
