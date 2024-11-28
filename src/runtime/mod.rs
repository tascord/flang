use {
    anyhow::anyhow,
    expr::ContextualExpr,
    scope::Scope,
    types::{Value, ValueType},
};

pub mod expr;
pub mod scope;
pub mod traits;
pub mod types;

pub mod _builtins;

pub fn step(node: ContextualExpr, s: &Scope) -> anyhow::Result<Option<Value>> {
    Ok(match node.0 {
        expr::Expr::Number(v) => Some(v.into()),
        expr::Expr::Boolean(v) => Some(v.into()),
        expr::Expr::String(v) => Some(v.into()),

        expr::Expr::Ident(v) => s.get(&v).map(|v| (*v).clone()),

        expr::Expr::Declaration { ident, typed, expr } => {
            let v = step(*expr, s)?.unwrap_or(Value::Undefined);

            if let Some(t) = typed {
                let ty = ValueType::from_str(&t, s).ok_or(anyhow!("Unknown type {}.", t))?;
                (ty.matches((&v).into(), s)).then_some(()).ok_or(anyhow!("Variable {} is not of type {}", ident, t))?;
            }

            s.declare(&ident, v);

            None
        }

        expr::Expr::Assignment { ident, expr } => {
            let v = step(*expr, s)?.unwrap_or(Value::Undefined);
            s.assign(&ident, v)?;

            None
        }

        expr::Expr::Print(expr) => {
            let v = step(*expr, s)?.unwrap_or(Value::Undefined);
            println!("Print => {v:?}");

            None
        }
    })
}
