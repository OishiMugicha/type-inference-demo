use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lexer,
    Parser,
    Inference,
    Evaluation,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lexer => "lexer",
            Self::Parser => "parser",
            Self::Inference => "型推論",
            Self::Evaluation => "評価器",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unimplemented,
    UnexpectedCharacter,
    IntegerOutOfRange,
    UnexpectedToken,
    UnboundVariable,
    TypeMismatch,
    InfiniteType,
    DivisionByZero,
    Overflow,
    NotCallable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub stage: Stage,
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(
        stage: Stage,
        kind: ErrorKind,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self {
            stage,
            kind,
            message: message.into(),
            span,
        }
    }

    pub fn unimplemented(stage: Stage) -> Self {
        Self::new(
            stage,
            ErrorKind::Unimplemented,
            format!("{stage}はまだ実装されていません。"),
            None,
        )
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.stage, self.message)?;
        if let Some(span) = self.span {
            write!(f, " (bytes {}..{})", span.start, span.end)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

pub type LangResult<T> = Result<T, Diagnostic>;
