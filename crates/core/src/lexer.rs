use crate::{
    error::{Diagnostic, LangResult, Stage},
    token::Token,
};

/// 空白を除くトークン列と、入力末尾のEofを返す。
pub fn lex(_source: &str) -> LangResult<Vec<Token>> {
    // TODO: docs/language.md と tests/lexer.rs の仕様を実装する。
    Err(Diagnostic::unimplemented(Stage::Lexer))
}
