use {
    anyhow::anyhow,
    expr::{ContextualExpr, Expr},
    itertools::Itertools,
    op::{get_dyadic, get_mondaic},
    pest::{iterators::Pair, Parser},
    pest_derive::Parser,
};

#[macro_use]
pub mod macros;
pub mod expr;
pub mod op;

#[derive(Parser)]
#[grammar = "./flang.pest"]
struct Flanger;

pub fn parse(s: &'static str) -> anyhow::Result<Vec<ContextualExpr>> {
    let pairs = Flanger::parse(Rule::program, &s)?;
    
    let mut ast: Vec<ContextualExpr> = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => {
                ast.push(build_ast_from_expr(pair)?);
            }
            _ => {}
        }
    }

    Ok(ast)
}

fn build_ast_from_expr(e: Pair<'static, Rule>) -> anyhow::Result<ContextualExpr> {
    match e.as_rule() {
        Rule::expr => build_ast_from_expr(e.clone().into_inner().next().unwrap()),
        Rule::terms => {
            let terms = e.clone().into_inner().map(|t| build_ast_from_term(t.clone())).collect::<Result<Vec<_>, _>>()?;
            Ok(match terms.len() {
                1 => terms.first().unwrap().clone(),
                _ => Expr::Terms(terms).context(e.as_span()),
            })
        }

        Rule::string | Rule::boolean | Rule::number | Rule::null | Rule::identifier => {
            build_ast_from_term(e.clone())
        }

        Rule::monadic => {
            let (verb, expr) = takes!(e, 2);
            build_mondaic(verb, build_ast_from_expr(expr)?)
        }
        Rule::dyadic => {
            let mut inner = e.clone().into_inner().rev();
            let mut right = build_ast_from_expr(inner.next().unwrap())?;

            for chunk in &inner.chunks(2) {
                let (verb, left) = chunk.collect_tuple().unwrap();
                let left = build_ast_from_expr(left.clone())?;
                right = build_dyadic(verb, left, right)?;
            }

            Ok(right)
        }

        Rule::var_decl => {
            if has!(e.clone(), "colon") {
                let (_, ident, _, typed, expr) = takes!(e.clone(), 5);
                Ok(Expr::Declaration {
                    ident: ident!(ident.clone())?,
                    typed: Some(typed.as_str().to_string()),
                    expr: Box::new(build_ast_from_expr(expr.clone())?),
                }
                .context(e.as_span()))
            } else {
                let (_, ident, expr) = takes!(e.clone(), 3);
                Ok(Expr::Declaration {
                    ident: ident!(ident.clone())?,
                    typed: None,
                    expr: Box::new(build_ast_from_expr(expr.clone())?),
                }
                .context(e.as_span()))
            }
        }

        Rule::var_assign => {
            let (ident, expr) = takes!(e.clone(), 2);
            Ok(Expr::Assignment { ident: ident!(ident.clone())?, expr: Box::new(build_ast_from_expr(expr.clone())?) }
                .context(e.as_span()))
        }

        Rule::fn_decl => {
            let (outline, block) = takes!(e.clone(), 2);
            let body = block
                .clone()
                .into_inner()
                .map(|t| build_ast_from_expr(t.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| anyhow!("Failed to parse function declaration"));

            let identifier = outline.clone().into_inner().find(|p| p.as_rule() == Rule::identifier).unwrap();
            let return_type =
                outline.clone().into_inner().last().filter(|p| p.as_rule() == Rule::typed).map(|n| n.as_str().to_string());
            let args = outline.clone().into_inner().find(|p| p.as_rule() == Rule::typed_args).map(|n| {
                n.into_inner()
                    .map(|n| {
                        let (a, _, b) = takes!(n, 3);
                        (a.as_str().to_string(), b.as_str().to_string())
                    })
                    .collect::<Vec<_>>()
            });

            Ok(Expr::FunctionDeclaration {
                ident: identifier.as_str().to_string(),
                args: args.unwrap_or_default(),
                return_type,
                body: body?,
            }
            .context(e.as_span()))
        }

        Rule::fn_call => {
            let mut args = e.clone().into_inner().collect::<Vec<_>>();
            let ident = ident!(args.first().unwrap().clone())?;
            if !args.is_empty() {
                args = args[1..].to_vec();
            }

            Ok(Expr::FunctionCall(
                ident,
                args.into_iter().map(|t| build_ast_from_term(t.clone().clone())).collect::<Result<Vec<_>, _>>()?,
            )
            .context(e.as_span()))
        }

        Rule::index => {
            let mut body = e.clone().into_inner();
            let item = build_ast_from_expr(body.next().unwrap().clone())?;
            let rest = body.map(|i| build_ast_from_expr(i.clone())).collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::Index(Box::new(item), rest).context(e.as_span()))
        }

        _ => {
            eprintln!("{:?} not yet implemented", e.as_rule());
            todo!()
        }
    }
}

fn build_ast_from_term(t: Pair<'static, Rule>) -> anyhow::Result<ContextualExpr> {
    match t.as_rule() {
        Rule::expr | Rule::fn_call => build_ast_from_expr(t.clone()).map(|e| e.0.clone()),
        Rule::identifier => Ok(Expr::Ident(String::from(t.as_str()))),

        Rule::string => Ok(Expr::String(t.as_str()[1..t.as_str().len() - 1].to_string())),
        Rule::boolean => {
            Ok(Expr::Boolean(t.as_str().trim().parse::<bool>().map_err(|er| anyhow!("Failed to parese boolean: {er:?}"))?))
        }
        Rule::number => {
            Ok(Expr::Number(t.as_str().trim().parse::<f64>().map_err(|er| anyhow!("Failed to parse number: {er:?}"))?))
        }
        Rule::null => Ok(Expr::Undefined),

        _ => {
            eprintln!("{:?} not yet implemented", t.as_rule());
            todo!()
        }
    }
    .map(|n| n.context(t.as_span()))
}

fn build_mondaic(pair: Pair<'static, Rule>, expr: ContextualExpr) -> anyhow::Result<ContextualExpr> {
    Ok(Expr::MondaicOp {
        verb: get_mondaic(pair.as_str().to_string()).ok_or(anyhow!("Failed to parse monad"))?,
        expr: Box::new(expr),
    }
    .context(pair.as_span()))
}

fn build_dyadic(pair: Pair<'static, Rule>, lhs: ContextualExpr, rhs: ContextualExpr) -> anyhow::Result<ContextualExpr> {
    Ok(Expr::DyadicOp {
        verb: get_dyadic(pair.as_str().to_string()).ok_or(anyhow!("Failed to parse dyad"))?,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
    .context(pair.as_span()))
}
