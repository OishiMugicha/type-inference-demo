use crate::{
    ast::Expr,
    error::{Diagnostic, LangResult, Stage},
    token::Token,
};

/// Eofまでを消費して、単一の式を返す。
pub fn parse(_tokens: &[Token]) -> LangResult<Expr> {
    // TODO: docs/language.md と tests/parser.rs の仕様を実装する。
    Err(Diagnostic::unimplemented(Stage::Parser))
}
