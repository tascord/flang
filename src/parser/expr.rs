use {super::op::{Dyadic, Mondaic}, enum_as_inner::EnumAsInner, pest::Span, std::ops::Deref};

#[derive(Debug, Clone)]
pub struct ContextualExpr(pub Expr, pub Span<'static>);
impl Deref for ContextualExpr {
    type Target = Expr;

    fn deref(&self) -> &Self::Target { &self.0 }
}

pub type BCExpr = Box<ContextualExpr>;

#[derive(Debug, Clone, EnumAsInner)]
pub enum Expr {
    Number(f64),
    Boolean(bool),
    String(String),
    Undefined,
    Terms(Vec<ContextualExpr>),

    Ident(String),
    Index(BCExpr, Vec<ContextualExpr>),

    FunctionCall(String, Vec<ContextualExpr>),
    FunctionDeclaration {
        ident: String,
        args: Vec<(String, String)>,
        return_type: Option<String>,
        body: Vec<ContextualExpr>,
    },

    Declaration { ident: String, typed: Option<String>, expr: BCExpr },
    Assignment { ident: String, expr: BCExpr },

    MondaicOp {
        verb: Mondaic,
        expr: Box<ContextualExpr>,
    },

    DyadicOp {
        verb: Dyadic,
        lhs: Box<ContextualExpr>,
        rhs: Box<ContextualExpr>,
    },

    Export(BCExpr),
    Import(String)
}


impl Expr {
    pub fn context(self, s: Span<'static>) -> ContextualExpr {
        ContextualExpr(self, s)
    }
}