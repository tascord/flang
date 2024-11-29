use {
    flang::*,
    parser::expr::{ContextualExpr, Expr},
    pest::Span,
    runtime::{scope::Scope, types::Value},
};

fn main() {
    let s = Span::new("", 0, 0).unwrap();

    let mut result = Value::Undefined.anonymous();
    let scope = Scope::new();

    let nodes = vec![
        ContextualExpr(
            Expr::Declaration {
                ident: "a".to_string(),
                typed: None,
                expr: Box::new(ContextualExpr(Expr::Number(123.0), s.clone())),
            },
            s.clone(),
        ),
        ContextualExpr(Expr::Print(Box::new(ContextualExpr(Expr::Ident("a".to_string()), s.clone()))), s.clone()),
    ];

    for n in nodes {
        if let Some(v) = runtime::step(n, &scope).unwrap() {
            result = v;
        };
    }

    println!("Result: {result:?}");
}
