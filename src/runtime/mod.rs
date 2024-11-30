use {
    crate::parser::expr::{self, ContextualExpr},
    _builtins::traits::default_impl,
    anyhow::anyhow,
    itertools::Itertools,
    scope::Scope,
    types::{
        function::{BasicFunction, Function, FunctionOutline},
        ContextualValue, Value, ValueType,
    },
};

pub mod scope;
pub mod traits;
pub mod types;

pub mod _builtins;

pub fn process(tree: Vec<ContextualExpr>, s: Option<&Scope>) -> anyhow::Result<Option<ContextualValue>> {
    let mut result = Value::Undefined.anonymous();

    let binding = Scope::new();
    default_impl(&binding);

    let scope = s.unwrap_or(&binding);
    for n in tree {
        if let Some(v) = step(n, &scope)? {
            result = v;
        };
    }

    Ok(Some(result))
}

pub fn step(node: ContextualExpr, s: &Scope) -> anyhow::Result<Option<ContextualValue>> {
    Ok(match node.0 {
        expr::Expr::Number(v) => Some(Value::from(v).context(node.1)),
        expr::Expr::Boolean(v) => Some(Value::from(v).context(node.1)),
        expr::Expr::String(v) => Some(Value::from(v).context(node.1)),
        expr::Expr::Undefined => Some(Value::Undefined.context(node.1)),

        expr::Expr::Ident(v) => s.get(&v).map(|v| (*v).clone().context(node.1)),

        expr::Expr::Declaration { ident, typed, expr } => {
            let v = step(*expr, s)?.unwrap_or(Value::Undefined.anonymous());

            if let Some(t) = typed {
                let ty = ValueType::from_str(&t, s).ok_or(anyhow!("Unknown type {}.", t))?;
                (ty.matches(&*v, s)).then_some(()).ok_or(anyhow!("Variable {} is not of type {}", ident, t))?;
            }

            s.declare(&ident, v.0);

            None
        }

        expr::Expr::Assignment { ident, expr } => {
            let v = step(*expr, s)?.unwrap_or(Value::Undefined.anonymous());
            s.assign(&ident, v.0)?;

            None
        }

        expr::Expr::Index(target, idx) => {
            let mut left = s.child_for_var(step(*target, s)?.unwrap().0);
            for right in idx {
                left = s.child_for_var(step(right.clone(), &left)?.ok_or(anyhow!("Index {:?} doesnt exist", right))?.0);
            }

            None
        }

        expr::Expr::FunctionCall(ident, args) => {
            let v = s
                .get(&ident)
                .map(|v| <Value as Clone>::clone(&v).into_function().ok())
                .flatten()
                .ok_or(anyhow!("No function exists with the name {ident}"))?;

            let mut args: Vec<ContextualValue> =
                args.iter().map(|v| step(v.clone(), s).map(|v| v.unwrap())).try_collect()?;

            if v.wants_self() {
                let a = s.container().ok_or(anyhow!("Function taking parameter 'self' cannot be called statically"))?;
                args.insert(0, <Value as Clone>::clone(&a).anonymous().clone());
            }

            println!("calling {} with args {:?}", ident, args);

            let found_ret = v.call(s, args)?;
            println!("found: {:?}", found_ret);
            
            found_ret

        }

        expr::Expr::FunctionDeclaration { ident, args, return_type, body } => {
            let f = BasicFunction {
                outline: FunctionOutline {
                    inputs: args
                        .into_iter()
                        .map(|(i, t)| ValueType::from_str(&t, s).ok_or(anyhow!("Unknown tyoe {t}")).map(|t| (i, t)))
                        .try_collect()?,
                    returns: match return_type {
                        Some(ty) => Some(ValueType::from_str(&ty, s).ok_or(anyhow!("Unknown type {ty}"))?),
                        None => None,
                    },
                },
                body,
            };

            s.declare(&ident, Value::Function(f.packaged()));

            None
        }

        expr::Expr::MondaicOp { verb, expr } => todo!(),
        expr::Expr::DyadicOp { verb, lhs, rhs } => todo!(),

        _ => todo!(),
    })
}
