use {
    crate::{project::source::SOURCES, sitter::Span},
    miette::{GraphicalReportHandler, LabeledSpan, NamedSource, Severity},
    std::{fmt::Display, process},
};

#[macro_use]
pub mod macros;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum FlangStage {
    PreProcessing = 0,
    Runtime = 1,
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
    pub source: Option<NamedSource<String>>,
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
        match &self.source {
            Some(s) => Some(s),
            None => None,
        }
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

pub trait Erroneous<T, E> {
    /// Runtime Error
    fn rt(self, s: impl Into<Span>) -> std::result::Result<T, Error>;

    /// Runtime anonymous error
    fn rta(self) -> std::result::Result<T, Error>;
}

pub trait ErroneousExt<T> {
    fn hint(self, h: &str) -> std::result::Result<T, Error>;
    fn fatal(self, f: bool) -> std::result::Result<T, Error>;
    fn code(self, c: &str) -> std::result::Result<T, Error>;
    fn unwrappers(self) -> T;
}

impl<T, E> Erroneous<T, E> for std::result::Result<T, E>
where
    E: Display,
{
    fn rt(self, s: impl Into<Span>) -> std::result::Result<T, Error> {
        let span: Span = s.into();
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error {
                stage: FlangStage::Runtime,
                error: e.to_string(),
                hint: None,
                fatal: false,
                code: None,
                bounds: span.byte_bounds,
                source: Some(NamedSource::new(
                    span.file_nameish(),
                    SOURCES.get_source(span.source_file).unwrap().to_string(),
                )),
            }),
        }
    }

    fn rta(self) -> std::result::Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(Error {
                stage: FlangStage::Runtime,
                error: e.to_string(),
                hint: None,
                fatal: false,
                code: None,
                bounds: (0, 0),
                source: None,
            }),
        }
    }
}

impl<T> ErroneousExt<T> for std::result::Result<T, Error> {
    fn hint(self, h: &str) -> std::result::Result<T, Error> {
        self.map_err(|mut e| {
            e.hint = Some(h.to_string());
            e
        })
    }

    fn fatal(self, f: bool) -> std::result::Result<T, Error> {
        self.map_err(|mut e| {
            e.fatal = f;
            e
        })
    }

    fn code(self, c: &str) -> std::result::Result<T, Error> {
        self.map_err(|mut e| {
            e.code = Some(c.to_string());
            e
        })
    }

    fn unwrappers(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let mut out = String::new();
                let _ = GraphicalReportHandler::default().render_report(&mut out, &e);
                println!("{}", out);
                process::exit(1)
            }
        }
    }
}

impl Span {
    #[must_use]
    pub fn as_error(self, err: &str) -> Error {
        Error {
            stage: FlangStage::Runtime,
            error: err.to_string(),
            hint: None,
            fatal: false,
            code: None,
            bounds: self.byte_bounds,
            source: Some(NamedSource::new(self.file_nameish(), SOURCES.get_source(self.source_file).unwrap().to_string())),
        }
    }
}
