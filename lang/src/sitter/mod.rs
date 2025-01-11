use {
    crate::{
        errors::Erroneous,
        project::{pack, source::SOURCES, Package, EXPORTS},
    },
    expr::{ContextualExpr, Expr},
    itertools::Itertools,
    op::get_dyadic,
    std::{path::Path, sync::Arc},
    tree_sitter::{Node, Parser},
    tree_sitter_language::LanguageFn,
};

pub mod expr;
pub mod op;

extern "C" {
    fn tree_sitter_flang() -> *const ();
}

/// The tree-sitter [`LanguageFn`][LanguageFn] for this grammar.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_flang) };
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const NODE_TYPES: &str = include_str!("../../../tree-sitter/src/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).expect("Error loading Flang parser");
    }
}

pub struct ParseContext {
    source_file: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Span {
    pub byte_bounds: (usize, usize),
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub text: String,
    pub source_file: String,
}

impl Span {
    pub fn anonymous() -> Span {
        Span {
            byte_bounds: Default::default(),
            start: Default::default(),
            end: Default::default(),
            text: Default::default(),
            source_file: Default::default(),
        }
    }

    pub fn file_nameish(&self) -> String {
        let mut folder = Path::new(&self.source_file).parent().unwrap();
        let mut depth = 0;

        loop {
            if let Some(_) =
                folder.read_dir().unwrap().find(|f| f.as_ref().map(|v| v.file_name() == "manifest.json").unwrap_or_default())
            {
                return Path::new(&self.source_file)
                    .display()
                    .to_string()
                    .replace(&folder.parent().unwrap().display().to_string(), "")
                    .split_at(1)
                    .1
                    .to_string();
            }

            if let Some(parent) = folder.parent() {
                folder = parent
            } else {
                break;
            }

            depth = depth + 1;
        }

        self.source_file.clone()
    }
}

impl ParseContext {
    pub fn span(self: &Arc<Self>, n: Node) -> Span {
        let byte_bounds = (n.start_byte(), n.end_byte());
        let start = (n.start_position().row, n.start_position().column);
        let end = (n.end_position().row, n.end_position().column);

        let text = match SOURCES.get_source(self.source_file.clone()) {
            Some(source) => String::from_utf8_lossy(&source.as_bytes()[byte_bounds.0..byte_bounds.1]).to_string(),
            None => "[Anonymous]".to_string(),
        };

        Span { byte_bounds, start, end, text, source_file: self.source_file.clone() }
    }
}

pub trait NodeExt {
    fn text(&self, pc: &Arc<ParseContext>) -> String;
}

impl NodeExt for Node<'_> {
    fn text(&self, pc: &Arc<ParseContext>) -> String {
        pc.clone().span(self.clone()).text
    }
}

pub fn parse(path: String) -> (Vec<ContextualExpr>, Vec<Span>) {
    let input = SOURCES.get_source(path.clone()).unwrap().to_string();

    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).expect("Error loading Flang grammar");

    let tree = parser.parse(input, None).unwrap();
    let mut cursor = tree.root_node().walk();

    let mut ast: Vec<ContextualExpr> = Vec::new();
    let mut errors: Vec<Span> = Vec::new();

    let context = Arc::new(ParseContext { source_file: path });

    for node in tree.root_node().children(&mut cursor) {
        if node.is_extra() {
            continue;
        }

        if node.is_error() {
            errors.push(context.span(node));
            continue;
        }

        ast.push(build_ast_from_expr(node, &context.clone()).unwrap());
    }

    (ast, errors)
}

fn build_ast_from_expr(node: Node, pc: &Arc<ParseContext>) -> crate::errors::Result<ContextualExpr> {
    let children = node.children(&mut node.walk()).collect::<Vec<_>>();
    // println!("[Expr] {} => {:#?}", node.grammar_name(), children.iter().map(|c| c.grammar_name()).collect::<Vec<_>>());

    Ok::<Expr, crate::errors::Error>(match node.grammar_name() {
        "thing" | "expr" => return build_ast_from_expr(node.child(0).unwrap(), pc),
        "string" | "boolean" | "number" | "null" | "identifier" => return build_ast_from_term(node, pc),

        "return" => Expr::Return(Box::new(build_ast_from_expr(children[1], pc)?)),
        "export" => Expr::Export(Box::new(build_ast_from_expr(children[1], &pc.clone())?)),

        "term" | "term_excl" | "terms" => {
            let terms = children.into_iter().map(|n| build_ast_from_term(n, pc)).collect::<Result<Vec<_>, _>>()?;
            match terms.len() {
                1 => terms[0].0.clone(),
                _ => Expr::Terms(terms),
            }
        }

        "uses" => {
            let mut cursor = node.walk();
            let children = node.children(&mut cursor);

            let mut imports = children.map(|c| pc.clone().span(c).text).collect::<Vec<_>>();
            let package = imports.pop().unwrap();
            let mut package = package.split("::").map(|s| s.to_string()).collect::<Vec<_>>();

            if *package[0] == "self".to_string() {
                package[0] = Package::from_file(pc.source_file.clone().into()).unwrap().name;
            }

            pack().dependent_package(package.clone()).unwrap().process().rt(pc.span(node))?;
            Expr::Import(EXPORTS.read().unwrap().get(&package.join("::")).unwrap().clone(), imports)
        }

        "fn_decl" => {
            let (outline, _, block) = children.into_iter().collect_tuple().unwrap();
            let outline = outline.children(&mut outline.walk()).collect::<Vec<_>>();

            let mut walk = block.walk();
            let body = block.children(&mut walk).skip(1);
            let count = body.len();

            let body = body.take(count - 1).map(|n| build_ast_from_expr(n, pc)).collect::<Result<Vec<_>, _>>()?;

            let return_type = outline.iter().filter(|n| n.grammar_name() == "typed").last().map(|n| n.text(pc));

            let args = outline
                .iter()
                .filter(|n| n.grammar_name() == "typed_args")
                .flat_map(|n| {
                    n.children(&mut n.walk())
                        .filter(|n| n.grammar_name() != "comma")
                        .map(|n| {
                            n.children(&mut n.walk())
                                .map(|n| n.text(pc).strip_prefix(": ").map(|s| s.to_string()).unwrap_or(n.text(pc)))
                                .collect_tuple()
                                .unwrap()
                        })
                        .collect::<Vec<(String, String)>>()
                })
                .collect::<Vec<_>>();

            Expr::FunctionDeclaration { args, return_type, body }
        }

        "fn_call" => {
            let (ident, _, args, _) = children.into_iter().collect_tuple().unwrap();
            let args: Vec<ContextualExpr> = args
                .children(&mut args.walk())
                .filter(|n| n.grammar_name() != "comma".to_string())
                .map(|n| build_ast_from_expr(n, pc))
                .collect::<Result<Vec<_>, _>>()?;

            Expr::FunctionCall(ident.text(pc), args)
        }

        "var_decl" => {
            if children.iter().any(|n| n.grammar_name() == "colon") {
                let (_, ident, _, typed, expr) = children.into_iter().collect_tuple().unwrap();
                Expr::Declaration {
                    ident: ident.text(&pc),
                    typed: Some(typed.text(&pc)),
                    expr: Box::new(build_ast_from_expr(expr.clone(), pc)?),
                }
            } else {
                let (_, ident, _, expr) = children.into_iter().collect_tuple().unwrap();
                Expr::Declaration {
                    ident: ident.text(&pc),
                    typed: None,
                    expr: Box::new(build_ast_from_expr(expr.clone(), pc)?),
                }
            }
        }

        "dyadic" => {
            let mut inner = children.into_iter().rev();
            let mut right = build_ast_from_expr(inner.next().unwrap(), pc)?;

            for chunk in &inner.chunks(2) {
                let (verb, left) = chunk.collect_tuple().unwrap();
                let left = build_ast_from_expr(left.clone(), pc)?;
                right = build_dyadic(verb, left, right, pc)?;
            }

            right.0
        }

        _ => {
            unimplemented!("Unimplemented expr: {:?}", node.grammar_name())
        }
    })
    .map(|n| n.context(pc.span(node)))
}

fn build_ast_from_term(node: Node<'_>, pc: &Arc<ParseContext>) -> crate::errors::Result<ContextualExpr> {
    let children = node.children(&mut node.walk()).collect::<Vec<_>>();
    // println!("[Term] {} => {:#?}", node.grammar_name(), children.iter().map(|c| c.grammar_name()).collect::<Vec<_>>());

    Ok::<Expr, crate::errors::Error>(match node.grammar_name() {
        "term" | "term_excl" => return build_ast_from_expr(children[0], pc),
        "expr" | "fn_call" => return build_ast_from_expr(node, pc),

        "literal" => return build_ast_from_term(children[0], pc),

        "null" => Expr::Undefined,
        "identifier" => Expr::Ident(node.text(pc)),
        "string" => Expr::String(node.text(pc)),
        "number" => Expr::Number(node.text(pc).parse().rt(pc.span(node))?),
        "boolean" => Expr::Boolean(node.text(pc).parse().rt(pc.span(node))?),

        _ => return build_ast_from_expr(node, pc),
    })
    .map(|n| n.context(pc.span(node)))
}

fn build_dyadic(
    node: Node<'_>,
    lhs: ContextualExpr,
    rhs: ContextualExpr,
    pc: &Arc<ParseContext>,
) -> crate::errors::Result<ContextualExpr> {
    Ok(Expr::DyadicOp {
        verb: get_dyadic(node.text(pc)).ok_or(anyhow::anyhow!("Failed to parse dyad")).rt(pc.span(node))?,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
    .context(pc.span(node)))
}
