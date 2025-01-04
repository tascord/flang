use std::{path::Path, sync::Arc};

use expr::{ContextualExpr, Expr};
use tree_sitter::{Node, Parser};
use tree_sitter_language::LanguageFn;

use crate::project::{source::SOURCES, Package};

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
    pub fn span(self: Arc<Self>, n: Node) -> Span {
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

pub fn parse(path: String) {
    let input = SOURCES.get_source(path.clone()).unwrap().to_string();

    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).expect("Error loading Flang grammar");

    let tree = parser.parse(input, None).unwrap();
    dbg!(&tree);
    let mut cursor = tree.root_node().walk();

    let mut ast: Vec<ContextualExpr> = Vec::new();
    let context = Arc::new(ParseContext { source_file: path });

    for node in tree.root_node().children(&mut cursor) {
        if node.is_extra() {
            continue;
        }
        ast.push(build_ast_from_expr(node, context.clone()).unwrap());
    }

    dbg!(ast);
}

fn build_ast_from_expr(node: Node, pc: Arc<ParseContext>) -> crate::errors::Result<ContextualExpr> {
    dbg!(node);
    Ok::<Expr, crate::errors::Error>(match node.grammar_name() {
        "thing" | "expr" => return build_ast_from_expr(node.child(0).unwrap(), pc),

        "uses" => {
            // let mut cursor = node.walk();
            // let children = node.children(&mut cursor);

            // let mut imports = children.map(|c| pc.clone().span(c).text).collect::<Vec<_>>();
            // let package = imports.pop().unwrap();
            // let mut package = package.split("::").map(|s| s.to_string()).collect::<Vec<_>>();

            // if *package[0] == "self".to_string() {
            //     package[0] = Package::from_file(pc.source_file.clone().into()).unwrap().name;
            // }

            // pack()
            //     .dependent_package(package.clone())
            //     .unwrap()
            //     .process()
            //     .unwrap()

            Expr::Undefined
        }

        "export" => Expr::Export(Box::new(build_ast_from_expr(node.child(0).unwrap(), pc.clone())?)),

        _ => {
            unimplemented!("Unimplemented expr: {:?}", node.grammar_name())
        }
    })
    .map(|n| n.context(pc.span(node)))
}
