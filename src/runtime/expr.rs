use pest::Span;

#[derive(Debug)]
pub struct ContextualExpr(pub Expr, pub Span<'static>);
pub type BCExpr = Box<ContextualExpr>;

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Boolean(bool),
    String(String),

    Ident(String),
    Declaration { ident: String, typed: Option<String>, expr: BCExpr },

    Assignment { ident: String, expr: BCExpr },

    Print(BCExpr),
}
