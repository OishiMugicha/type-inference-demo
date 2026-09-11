use crate::{
    ast::Expr,
    error::{Diagnostic, LangResult, Stage},
    types::Type,
};

/// 空の型環境で、式の主型を推論する。型変数の番号は自由。
pub fn infer(_expr: &Expr) -> LangResult<Type> {
    // TODO: docs/language.md と tests/inference.rs の仕様を実装する。
    Err(Diagnostic::unimplemented(Stage::Inference))
}
