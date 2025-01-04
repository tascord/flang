use {
    crate::{
        errors::Erroneous,
       
        project::export, sitter::{expr::{self, ContextualExpr}, op::Dyadic},
    },
    _builtins::default_impl,
    anyhow::anyhow,
    itertools::Itertools,
    scope::Scope,
    std::path::PathBuf,
    types::{
        function::{BasicFunction, Function, FunctionOutline},
        ContextualValue, Value, ValueType,
    },
};

pub mod _builtins;
pub mod scope;
pub mod traits;
pub mod types;

pub fn process(
    tree: Vec<ContextualExpr>,
    s: Option<&Scope>,
    p: Option<PathBuf>,
) -> crate::errors::Result<Option<ContextualValue>> {
    let mut result = Value::Undefined.anonymous();

    let binding = Scope::new();
    default_impl(&binding);

    let scope = s.unwrap_or(&binding);
    for n in tree {
        if let Some(v) = step(n, &scope, &p)? {
            result = v;
        };
    }

    Ok(Some(result))
}

pub fn step(node: ContextualExpr, s: &Scope, p: &Option<PathBuf>) -> Result<Option<ContextualValue>, crate::errors::Error> {
    Ok(match node.0 {
        expr::Expr::Number(v) => Some(Value::from(v).context(node.1.clone())),
        expr::Expr::Boolean(v) => Some(Value::from(v).context(node.1.clone())),
        expr::Expr::String(v) => Some(Value::from(v).context(node.1.clone())),
        expr::Expr::Undefined => Some(Value::Undefined.context(node.1.clone())),

        expr::Expr::Ident(v) => s.get(&v).map(|v| (*v).clone().context(node.1.clone())),

        expr::Expr::Declaration { ident, typed, expr } => {
            let v = step(*expr, s, p)?.unwrap();

            if let Some(t) = typed {
                let ty = ValueType::from_str(&t, s).ok_or(anyhow!("Unknown type {}.", t)).rt(node.1.clone())?;
                (ty.matches(&*v, s))
                    .then_some(())
                    .ok_or(anyhow!("Variable {} is not of type {}", ident, t))
                    .rt(node.1.clone())?;
            }

            s.declare(&ident, v.0.clone());
            Some(v)
        }

        expr::Expr::Assignment { ident, expr } => {
            let v = step(*expr, s, p)?.unwrap_or(Value::Undefined.anonymous());
            s.assign(&ident, v.0.clone()).rt(node.1.clone())?;
            Some(v)
        }

        expr::Expr::Index(target, idx) => {
            let mut scope = s.child_for_var(step(*target, s, p)?.unwrap().0);
            let right = idx
                .into_iter()
                .fold(anyhow::Result::<ContextualValue>::Ok(Value::Undefined.anonymous()), |_, b| {
                    let right = step(b.clone(), &scope, p)?.ok_or(anyhow!("Index {:?} doesnt exist", b))?;
                    scope = s.child_for_var(right.0.clone());
                    Ok(right)
                })
                .rt(node.1.clone())?;

            Some(right)
        }

        expr::Expr::FunctionCall(ident, args) => {
            let v = s
                .get(&ident)
                .map(|v| <Value as Clone>::clone(&v).into_function().ok())
                .flatten()
                .ok_or(anyhow!("No function exists with the name {ident}"))
                .rt(node.1.clone())?;

            let mut args: Vec<ContextualValue> =
                args.iter().map(|v| step(v.clone(), s, p).map(|v| v.unwrap())).try_collect()?;

            if v.wants_self() {
                let a = s
                    .container()
                    .ok_or(anyhow!("Function taking parameter 'self' cannot be called statically"))
                    .rt(node.1.clone())?;
                args.insert(0, <Value as Clone>::clone(&a).anonymous().clone());
            }

            let found_ret = v.call(s, args).rt(node.1.clone())?;
            found_ret
        }

        expr::Expr::FunctionDeclaration { args, return_type, body } => {
            let f = BasicFunction {
                outline: FunctionOutline {
                    inputs: args
                        .into_iter()
                        .map(|(i, t)| {
                            ValueType::from_str(&t, s).ok_or(anyhow!("Unknown type {t}")).rt(node.1.clone()).map(|t| (i, t))
                        })
                        .try_collect()?,
                    returns: match return_type {
                        Some(ty) => {
                            Some(ValueType::from_str(&ty, s).ok_or(anyhow!("Unknown type {ty}")).rt(node.1.clone())?)
                        }
                        None => None,
                    },
                },
                body,
            };

            Some(Value::Function(f.packaged()).context(node.1.clone()))
        }

        expr::Expr::MondaicOp { .. } => todo!(),

        expr::Expr::DyadicOp { verb, lhs, rhs } => {
            let left = step(*lhs, s, p)?.unwrap();
            let right = step(*rhs, s, p)?.unwrap();

            if !<Value as Into<ValueType>>::into(left.0.clone()).matches(&right, s) {
                return Err(anyhow!("Can't perform dyadic operations on differing types.")).rt(node.1.clone());
            }

            let trait_name = match verb {
                Dyadic::Add => "Add",
                _ => todo!(),
            }
            .to_string();

            s.get_traits_for(left.0.clone())
                .into_iter()
                .find(|t| t.def.name == trait_name)
                .unwrap()
                .get_function(&trait_name.to_lowercase())
                .unwrap()
                .call(s, vec![left, right])
                .rt(node.1.clone())?
        }

        expr::Expr::Return(expr) => {
            let value = step(*expr, s, p)?.unwrap_or(Value::Undefined.anonymous());
            Some(Value::Return(Box::new(value.0)).context(value.1))
        }

        expr::Expr::Export(expr) => {
            s.use_export(export(
                p.clone().ok_or(anyhow!("Can't export in a non-path based environment")).rta()?.display().to_string(),
            ));

            let value = step(*expr, s, p)?.unwrap_or(Value::Undefined.anonymous());

            s.clear_export();

            Some(value)
        }

        expr::Expr::Import(scope, imports) => {
            if imports.len() == 0 {
                s.absorb(scope);
            } else {
                imports.into_iter().for_each(|i| s.absorb_named(scope.clone(), i));
            }

            None
        }

        _ => todo!(),
    })
}
