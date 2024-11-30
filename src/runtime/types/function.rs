use {
    super::{ContextualValue, Value, ValueType},
    crate::{
        parser::expr::ContextualExpr,
        runtime::{process, scope::Scope},
    },
    anyhow::bail,
    std::{fmt::Debug, sync::Arc},
};

pub trait Function: Sync + Send + Debug {
    fn call(&self, scope: &Scope, inputs: Vec<ContextualValue>) -> anyhow::Result<Option<ContextualValue>>;
    fn outline(&self) -> FunctionOutline;
    fn packaged(self) -> Arc<Box<dyn Function>>;
    fn wants_self(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct BasicFunction {
    pub outline: FunctionOutline,
    pub body: Vec<ContextualExpr>,
}

impl Function for BasicFunction {
    fn call(&self, scope: &Scope, inputs: Vec<ContextualValue>) -> anyhow::Result<Option<ContextualValue>> {
        let s = scope.child();
        self.outline
            .inputs
            .iter()
            .zip(inputs)
            .map(|((ident, ty), v)| -> anyhow::Result<()> {
                if !ty.matches(&v.0, &s) {
                    bail!("Mismatching types for fn args");
                }

                s.declare(&ident, v.0);
                Ok(())
            })
            .collect::<anyhow::Result<Vec<()>>>()?;

        process(self.body.clone(), Some(&s))
    }

    fn outline(&self) -> FunctionOutline {
        self.outline.clone()
    }

    fn packaged(self) -> Arc<Box<dyn Function>> {
        Arc::new(Box::new(self) as Box<dyn Function>)
    }

    fn wants_self(&self) -> bool {
        self.outline.inputs.first().map(|v| &v.0 == "self").unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<ContextualValue>) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    pub outline: FunctionOutline,
    pub handler: Arc<Box<T>>,
}

impl<T> Debug for BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<ContextualValue>) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Builtin Function]")
    }
}

impl<T> Function for BuiltinFunction<T>
where
    T: Fn(&Scope, Vec<ContextualValue>) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    fn call(&self, scope: &Scope, inputs: Vec<ContextualValue>) -> anyhow::Result<Option<ContextualValue>> {
        let call_ret = (self.handler.clone())(scope, inputs);
        println!("cret: {:?}", call_ret);
        Ok(call_ret)
    }

    fn outline(&self) -> FunctionOutline {
        self.outline.clone()
    }

    fn packaged(self) -> Arc<Box<dyn Function>> {
        Arc::new(Box::new(self) as Box<dyn Function>)
    }

    fn wants_self(&self) -> bool {
        self.outline.inputs.first().map(|v| &v.0 == "self").unwrap_or(false)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FunctionOutline {
    pub inputs: Vec<(String, ValueType)>,
    pub returns: Option<ValueType>,
}
