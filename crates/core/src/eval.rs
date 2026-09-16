use std::rc::Rc;

use crate::{
    ast::{BinaryOp, Expr, ExprKind as E},
    error::{Diagnostic, ErrorKind, LangResult, Stage},
    value::{Closure, Value, ValueEnv},
};

/// 空の値環境で式を評価する。型推論の実装には依存しない。
pub fn eval(expr: &Expr) -> LangResult<Value> {
    evaluate(expr, &ValueEnv::new())
}

fn error(expr: &Expr, kind: ErrorKind, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Stage::Evaluation, kind, message, Some(expr.span))
}

fn integer(value: Value, expr: &Expr) -> LangResult<i64> {
    match value {
        Value::Int(value) => Ok(value),
        _ => Err(error(expr, ErrorKind::TypeMismatch, "整数が必要です。")),
    }
}

fn checked(value: Option<i64>, expr: &Expr) -> LangResult<Value> {
    value.map(Value::Int).ok_or_else(|| {
        error(
            expr,
            ErrorKind::Overflow,
            "整数演算が許容範囲を超えました。",
        )
    })
}

fn evaluate(expr: &Expr, env: &ValueEnv) -> LangResult<Value> {
    match &expr.kind {
        E::Int(value) => Ok(Value::Int(*value)),
        E::Bool(value) => Ok(Value::Bool(*value)),
        E::Var(name) => env.get(name).cloned().ok_or_else(|| {
            error(
                expr,
                ErrorKind::UnboundVariable,
                format!("未束縛の変数です: {name}"),
            )
        }),
        E::Negate(inner) => {
            let value = integer(evaluate(inner, env)?, inner)?;
            checked(value.checked_neg(), expr)
        }
        E::Binary { op, left, right } => {
            // 型の確認より先に、両辺を左から順に評価する。
            let a = evaluate(left, env)?;
            let b = evaluate(right, env)?;
            let a = integer(a, left)?;
            let b = integer(b, right)?;
            match op {
                BinaryOp::Add => checked(a.checked_add(b), expr),
                BinaryOp::Subtract => checked(a.checked_sub(b), expr),
                BinaryOp::Multiply => checked(a.checked_mul(b), expr),
                BinaryOp::Divide if b == 0 => Err(error(
                    expr,
                    ErrorKind::DivisionByZero,
                    "ゼロで除算できません。",
                )),
                BinaryOp::Divide => checked(a.checked_div(b), expr),
                BinaryOp::Less => Ok(Value::Bool(a < b)),
            }
        }
        E::If {
            condition,
            then_branch,
            else_branch,
        } => match evaluate(condition, env)? {
            Value::Bool(true) => evaluate(then_branch, env),
            Value::Bool(false) => evaluate(else_branch, env),
            _ => Err(error(
                condition,
                ErrorKind::TypeMismatch,
                "条件には真偽値が必要です。",
            )),
        },
        E::Let { name, value, body } => {
            let value = evaluate(value, env)?;
            let mut local = env.clone();
            local.insert(name.clone(), value);
            evaluate(body, &local)
        }
        E::Fun { parameter, body } => Ok(Value::Closure(Rc::new(Closure {
            parameter: parameter.clone(),
            body: body.as_ref().clone(),
            environment: env.clone(),
        }))),
        E::Apply { function, argument } => {
            let function_value = evaluate(function, env)?;
            let argument_value = evaluate(argument, env)?;
            let Value::Closure(closure) = function_value else {
                return Err(error(
                    function,
                    ErrorKind::NotCallable,
                    "関数以外の値は呼び出せません。",
                ));
            };
            let mut local = closure.environment.clone();
            local.insert(closure.parameter.clone(), argument_value);
            evaluate(&closure.body, &local)
        }
    }
}
