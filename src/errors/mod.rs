use std::{fmt::Display, process};

use miette::{LabeledSpan, Severity};

#[macro_use]
pub mod macros;

#[derive(Debug, Clone, Copy)]
pub enum FlangStage {
    PreProcessing,
    Runtime,
}

impl Display for FlangStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FlangStage::PreProcessing => "Pre-Processing",
                FlangStage::Runtime => "Runtime",
            }
        )
    }
}

#[derive(Debug)]
pub struct Error {
    pub stage: FlangStage,
    pub error: String,
    pub hint: Option<String>,
    pub fatal: bool,
    pub code: Option<String>,
    pub bounds: (usize, usize),
}

impl Error {
    pub fn exec(self) {
        if self.fatal {
            process::exit(1)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{code}Error in {stage}: {error:?}{hint}",
            stage = self.stage,
            error = self.error,
            hint = self.hint.clone().map(|m| format!("{m}\n")).unwrap_or_default(),
            code = self.code.clone().map(|c| format!("{c} ")).unwrap_or_default()
        )
    }
}

impl std::error::Error for Error {}
impl miette::Diagnostic for Error {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.code.clone().map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.fatal {
            true => Severity::Error,
            false => Severity::Warning,
        })
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.hint.clone().map(|c| Box::new(c) as Box<dyn std::fmt::Display>)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        None // TODO
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(
            vec![LabeledSpan::new(Some(self.error.clone()), self.bounds.0, self.bounds.1 - self.bounds.0)].into_iter(),
        ))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        None
    }
}
