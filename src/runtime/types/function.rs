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

pub fn declare(f: Arc<Box<dyn Function + 'static>>, s: &Scope, i: Vec<ContextualValue>) -> anyhow::Result<Scope> {
    let s = s.child_for_var(Value::Function(f.clone()));
    f.outline()
        .inputs
        .iter()
        .zip(i)
        .map(|((ident, ty), v)| -> anyhow::Result<()> {
            if !ty.matches(&v.0, &s) {
                bail!("Mismatching types for fn args {:?} != {}", ValueType::from(v.0.into()), match ty {
                    ValueType::This => format!(
                        "Self[ {:?} ]",
                        s.container()
                            .map(|v| <Value as Into<ValueType>>::into((*v).clone()))
                            .unwrap_or(ValueType::Undefined)
                    ),
                    ty => format!("{ty:?}"),
                });
            }

            s.declare(&ident, v.0);
            Ok(())
        })
        .collect::<anyhow::Result<Vec<()>>>()?;

    Ok(s)
}

#[derive(Clone)]
pub struct BasicFunction {
    pub outline: FunctionOutline,
    pub body: Vec<ContextualExpr>,
}

impl Debug for BasicFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Function ({}) -> {}]",
            self.outline.inputs.iter().map(|v| format!("{:?}", v.1)).collect::<Vec<_>>().join(", "),
            match &self.outline.returns {
                Some(ty) => format!("{:?}", ty),
                None => "void".to_string(),
            }
        )
    }
}

impl Function for BasicFunction {
    fn call(&self, scope: &Scope, inputs: Vec<ContextualValue>) -> anyhow::Result<Option<ContextualValue>> {
        process(self.body.clone(), Some(&declare(self.clone().packaged(), scope, inputs)?))
    }

    fn outline(&self) -> FunctionOutline { self.outline.clone() }

    fn packaged(self) -> Arc<Box<dyn Function>> { Arc::new(Box::new(self) as Box<dyn Function>) }

    fn wants_self(&self) -> bool { self.outline.inputs.first().map(|v| &v.0 == "self").unwrap_or(false) }
}

#[derive(Clone)]
pub struct BuiltinFunction<T>
where
    T: Fn(&Scope) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    pub outline: FunctionOutline,
    pub handler: Arc<Box<T>>,
}

impl<T> Debug for BuiltinFunction<T>
where
    T: Fn(&Scope) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Builtin Function ({}) -> {}]",
            self.outline.inputs.iter().map(|v| format!("{:?}", v.1)).collect::<Vec<_>>().join(", "),
            match &self.outline.returns {
                Some(ty) => format!("{:?}", ty),
                None => "void".to_string(),
            }
        )
    }
}

impl<T> Function for BuiltinFunction<T>
where
    T: Fn(&Scope) -> Option<ContextualValue> + Sync + Send + Clone + 'static,
{
    fn call(&self, scope: &Scope, inputs: Vec<ContextualValue>) -> anyhow::Result<Option<ContextualValue>> {
        Ok((self.handler.clone())(&declare(self.clone().packaged(), scope, inputs)?))
    }

    fn outline(&self) -> FunctionOutline { self.outline.clone() }

    fn packaged(self) -> Arc<Box<dyn Function>> { Arc::new(Box::new(self) as Box<dyn Function>) }

    fn wants_self(&self) -> bool { self.outline.inputs.first().map(|v| &v.0 == "self").unwrap_or(false) }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FunctionOutline {
    pub inputs: Vec<(String, ValueType)>,
    pub returns: Option<ValueType>,
}
