mod support;
use support::*;
use tiny_ml_core::{
    ast::BinaryOp as Op,
    error::{ErrorKind, Stage},
    eval,
    value::Value,
};

fn assert_int(expression: tiny_ml_core::ast::Expr, expected: i64) {
    assert!(matches!(eval(&expression).unwrap(), Value::Int(actual) if actual == expected));
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn arithmetic_and_comparison() {
    for (op, left, right, expected) in [
        (Op::Add, 2, 3, 5),
        (Op::Subtract, 2, 3, -1),
        (Op::Multiply, 2, 3, 6),
        (Op::Divide, -7, 2, -3),
    ] {
        assert_int(binary(op, int(left), int(right)), expected);
    }
    assert_int(negate(int(3)), -3);
    assert!(matches!(
        eval(&binary(Op::Less, int(1), int(2))).unwrap(),
        Value::Bool(true)
    ));
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn conditional_only_evaluates_selected_branch() {
    assert_int(
        branch(boolean(true), int(42), binary(Op::Divide, int(1), int(0))),
        42,
    );
    assert_int(branch(boolean(false), var("missing"), int(7)), 7);
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn binding_shadowing_and_closures() {
    assert_int(bind("x", int(1), bind("x", int(2), var("x"))), 2);
    assert_int(apply(fun("x", var("x")), int(42)), 42);
    assert!(matches!(
        eval(&fun("x", var("x"))).unwrap(),
        Value::Closure(_)
    ));
    let expression = bind(
        "x",
        int(10),
        bind(
            "f",
            fun("y", binary(Op::Add, var("x"), var("y"))),
            bind("x", int(100), apply(var("f"), int(2))),
        ),
    );
    assert_int(expression, 12);
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn higher_order_closure_retains_parameter() {
    let expression = apply(
        apply(
            fun("x", fun("y", binary(Op::Add, var("x"), var("y")))),
            int(40),
        ),
        int(2),
    );
    assert_int(expression, 42);
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn call_by_value_and_left_to_right() {
    let zero_division = binary(Op::Divide, int(1), int(0));
    assert_eq!(
        eval(&apply(fun("x", int(42)), zero_division.clone()))
            .unwrap_err()
            .kind,
        ErrorKind::DivisionByZero
    );
    assert_eq!(
        eval(&apply(var("missing"), zero_division.clone()))
            .unwrap_err()
            .kind,
        ErrorKind::UnboundVariable
    );
    assert_eq!(
        eval(&binary(Op::Add, zero_division, var("missing")))
            .unwrap_err()
            .kind,
        ErrorKind::DivisionByZero
    );
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn overflow_and_zero_division() {
    for expression in [
        binary(Op::Add, int(i64::MAX), int(1)),
        binary(Op::Subtract, int(i64::MIN), int(1)),
        binary(Op::Multiply, int(i64::MAX), int(2)),
        binary(Op::Divide, int(i64::MIN), int(-1)),
        negate(int(i64::MIN)),
    ] {
        assert_eq!(eval(&expression).unwrap_err().kind, ErrorKind::Overflow);
    }
    assert_eq!(
        eval(&binary(Op::Divide, int(1), int(0))).unwrap_err().kind,
        ErrorKind::DivisionByZero
    );
}

#[test]
#[ignore = "exercise: evaluation; parser不要"]
fn runtime_errors_without_type_inference() {
    let error = eval(&var("missing")).unwrap_err();
    assert_eq!(error.stage, Stage::Evaluation);
    assert_eq!(error.kind, ErrorKind::UnboundVariable);
    assert_eq!(
        eval(&apply(int(1), int(2))).unwrap_err().kind,
        ErrorKind::NotCallable
    );
    for expression in [
        binary(Op::Add, int(1), boolean(true)),
        branch(int(1), int(2), int(3)),
        negate(boolean(true)),
    ] {
        assert_eq!(eval(&expression).unwrap_err().kind, ErrorKind::TypeMismatch);
    }
    assert_eq!(
        eval(&bind("x", var("x"), var("x"))).unwrap_err().kind,
        ErrorKind::UnboundVariable
    );
}
