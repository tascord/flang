use pest::Span;

#[macro_use]
pub mod macros;

#[derive(Clone, Copy)]
pub enum FlangStage {
    PreProcessing,
    Runtime,
}

pub struct Error {
    stage: FlangStage,
    error: String,
    hint: Option<String>,
}

pub fn get_source(s: Span) {
    s
}