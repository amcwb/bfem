use std::fmt::Display;

use miette::{
    Diagnostic, GraphicalReportHandler, GraphicalTheme, NarratableReportHandler, Report, SourceSpan,
};
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub enum BFErrors {
    RuntimeError,
}

#[derive(Error, Debug)]
pub struct BFError {
    pub error: BFErrors,
    pub message: String,
}

impl Display for BFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error, self.message)
    }
}

impl BFError {
    pub fn into_detailed(&self, highlight: SourceSpan) -> BFDetailedError {
        BFDetailedError {
            error: self.error,
            message: self.message.clone(),
            highlight,
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Oh no")]
#[diagnostic()]
pub struct BFDetailedError {
    error: BFErrors,
    message: String,

    #[label("here")]
    highlight: SourceSpan,
}

impl BFError {
    pub fn new(error: BFErrors, message: String) -> Self {
        Self { error, message }
    }
}

impl BFDetailedError {
    pub fn new(error: BFErrors, message: String, highlight: SourceSpan) -> Self {
        Self {
            error,
            message,
            highlight,
        }
    }
}

pub fn fmt_report(diag: Report) -> String {
    let mut out = String::new();
    // Mostly for dev purposes.
    if std::env::var("STYLE").is_ok() {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .with_width(80)
            .with_footer("this is a footer".into())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else if std::env::var("NARRATED").is_ok() {
        NarratableReportHandler::new()
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else if let Ok(w) = std::env::var("REPLACE_TABS") {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(80)
            .tab_width(w.parse().expect("Invalid tab width."))
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    } else {
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(80)
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    };
    out
}
