use crate::{
    ast::Expr,
    error::{Diagnostic, LangResult, Stage},
    value::Value,
};

/// 空の値環境で式を評価する。型推論の実装には依存しない。
pub fn eval(_expr: &Expr) -> LangResult<Value> {
    // TODO: docs/language.md と tests/evaluation.rs の仕様を実装する。
    Err(Diagnostic::unimplemented(Stage::Evaluation))
}
