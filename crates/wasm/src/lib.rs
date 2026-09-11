//! Web向けの接続部分。言語処理の実装はtiny-ml-coreに置く。
use std::fmt::{Debug, Display};

use serde::Serialize;
use tiny_ml_core::{
    ast::Expr,
    error::{ErrorKind, LangResult},
    token::Token,
    types::Type,
    value::Value,
};
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StageResult {
    Success { output: String },
    Unimplemented { message: String },
    Error { message: String },
    Skipped { message: String },
}

impl StageResult {
    fn skipped(message: &str) -> Self {
        Self::Skipped {
            message: message.into(),
        }
    }

    fn from_result<T>(result: &LangResult<T>, format: impl FnOnce(&T) -> String) -> Self {
        match result {
            Ok(value) => Self::Success {
                output: format(value),
            },
            Err(error) if error.kind == ErrorKind::Unimplemented => Self::Unimplemented {
                message: error.to_string(),
            },
            Err(error) => Self::Error {
                message: error.to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    pub tokens: StageResult,
    pub ast: StageResult,
    pub inferred_type: StageResult,
    pub value: StageResult,
}

fn debug_output(value: &impl Debug) -> String {
    format!("{value:#?}")
}

fn display_output(value: &impl Display) -> String {
    value.to_string()
}

fn analyze_with(
    source: &str,
    lex: impl FnOnce(&str) -> LangResult<Vec<Token>>,
    parse: impl FnOnce(&[Token]) -> LangResult<Expr>,
    infer: impl FnOnce(&Expr) -> LangResult<Type>,
    eval: impl FnOnce(&Expr) -> LangResult<Value>,
) -> AnalysisReport {
    let tokens = lex(source);
    let mut report = AnalysisReport {
        tokens: StageResult::from_result(&tokens, debug_output),
        ast: StageResult::skipped("lexerが成功すると実行できます。"),
        inferred_type: StageResult::skipped("parserが成功すると実行できます。"),
        value: StageResult::skipped("parserが成功すると実行できます。"),
    };
    if let Ok(tokens) = tokens {
        let expr = parse(&tokens);
        report.ast = StageResult::from_result(&expr, debug_output);
        if let Ok(expr) = expr {
            report.inferred_type = StageResult::from_result(&infer(&expr), display_output);
            report.value = StageResult::from_result(&eval(&expr), display_output);
        }
    }
    report
}

/// JSON文字列を返す。数値や内部ASTは表示文字列に変換するため、JSの整数精度に依存しない。
#[wasm_bindgen]
pub fn analyze(source: &str) -> Result<String, JsValue> {
    let report = analyze_with(
        source,
        tiny_ml_core::lex,
        tiny_ml_core::parse,
        tiny_ml_core::infer,
        tiny_ml_core::eval,
    );
    serde_json::to_string(&report).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_ml_core::{
        ast::ExprKind,
        error::{Diagnostic, Stage},
        span::Span,
    };

    fn fixture_expr() -> Expr {
        Expr::new(ExprKind::Int(42), Span::new(0, 2))
    }

    #[test]
    fn lexer_error_skips_all_dependents() {
        let report = analyze_with(
            "@",
            |_| {
                Err(Diagnostic::new(
                    Stage::Lexer,
                    ErrorKind::UnexpectedCharacter,
                    "invalid",
                    Some(Span::new(0, 1)),
                ))
            },
            |_| panic!("parser must not run"),
            |_| panic!("inference must not run"),
            |_| panic!("evaluation must not run"),
        );
        assert!(matches!(report.tokens, StageResult::Error { .. }));
        assert!(matches!(report.ast, StageResult::Skipped { .. }));
        assert!(matches!(report.inferred_type, StageResult::Skipped { .. }));
        assert!(matches!(report.value, StageResult::Skipped { .. }));
    }

    #[test]
    fn parser_unimplemented_preserves_tokens() {
        let report = analyze_with(
            "42",
            |_| Ok(vec![]),
            |_| Err(Diagnostic::unimplemented(Stage::Parser)),
            |_| panic!("inference must not run"),
            |_| panic!("evaluation must not run"),
        );
        assert!(matches!(report.tokens, StageResult::Success { .. }));
        assert!(matches!(report.ast, StageResult::Unimplemented { .. }));
        assert!(matches!(report.value, StageResult::Skipped { .. }));
    }

    #[test]
    fn inference_failure_does_not_prevent_evaluation() {
        for kind in [ErrorKind::Unimplemented, ErrorKind::TypeMismatch] {
            let report = analyze_with(
                "42",
                |_| Ok(vec![]),
                |_| Ok(fixture_expr()),
                |_| Err(Diagnostic::new(Stage::Inference, kind, "fixture", None)),
                |_| Ok(Value::Int(42)),
            );
            assert!(matches!(report.ast, StageResult::Success { .. }));
            assert!(matches!(report.value, StageResult::Success { output } if output == "42"));
        }
    }

    #[test]
    fn transport_preserves_large_integers_and_statuses() {
        let report = analyze_with(
            "42",
            |_| Ok(vec![]),
            |_| Ok(fixture_expr()),
            |_| Ok(Type::Int),
            |_| Ok(Value::Int(i64::MAX)),
        );
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).unwrap()).unwrap();
        assert_eq!(json["inferred_type"]["status"], "success");
        assert_eq!(json["value"]["output"], "9223372036854775807");
    }
}
