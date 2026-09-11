mod support;
use support::*;
use tiny_ml_core::{
    ast::BinaryOp as Op,
    error::{ErrorKind, Stage},
    infer,
    types::Type as T,
};

#[test]
#[ignore = "exercise: inference; parser不要"]
fn literals_arithmetic_comparison_and_negation() {
    assert_type_eq(infer(&int(42)).unwrap(), T::Int);
    assert_type_eq(infer(&boolean(true)).unwrap(), T::Bool);
    assert_type_eq(infer(&binary(Op::Add, int(1), int(2))).unwrap(), T::Int);
    assert_type_eq(infer(&binary(Op::Less, int(1), int(2))).unwrap(), T::Bool);
    assert_type_eq(infer(&negate(int(3))).unwrap(), T::Int);
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn identity_and_higher_order_function() {
    assert_type_eq(
        infer(&fun("x", var("x"))).unwrap(),
        arrow(T::Var(0), T::Var(0)),
    );
    let expression = fun("f", fun("x", apply(var("f"), var("x"))));
    assert_type_eq(
        infer(&expression).unwrap(),
        arrow(arrow(T::Var(0), T::Var(1)), arrow(T::Var(0), T::Var(1))),
    );
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn let_polymorphism() {
    let expression = bind(
        "id",
        fun("x", var("x")),
        bind(
            "a",
            apply(var("id"), int(42)),
            apply(var("id"), boolean(true)),
        ),
    );
    assert_type_eq(infer(&expression).unwrap(), T::Bool);
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn lambda_parameter_is_monomorphic() {
    let expression = fun(
        "f",
        bind(
            "a",
            apply(var("f"), int(42)),
            apply(var("f"), boolean(true)),
        ),
    );
    assert_eq!(
        infer(&expression).unwrap_err().kind,
        ErrorKind::TypeMismatch
    );
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn let_does_not_generalize_environment_variables() {
    // fun x -> let y = x in let a = y 1 in y true
    let expression = fun(
        "x",
        bind(
            "y",
            var("x"),
            bind("a", apply(var("y"), int(1)), apply(var("y"), boolean(true))),
        ),
    );
    assert_eq!(
        infer(&expression).unwrap_err().kind,
        ErrorKind::TypeMismatch
    );
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn conditional_types_and_errors() {
    assert_type_eq(
        infer(&branch(boolean(true), int(1), int(2))).unwrap(),
        T::Int,
    );
    for expression in [
        branch(int(1), int(2), int(3)),
        branch(boolean(true), int(1), boolean(false)),
        binary(Op::Add, int(1), boolean(true)),
        apply(int(1), int(2)),
    ] {
        assert_eq!(
            infer(&expression).unwrap_err().kind,
            ErrorKind::TypeMismatch
        );
    }
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn unbound_variables_and_nonrecursive_let() {
    for expression in [var("missing"), bind("x", var("x"), var("x"))] {
        let error = infer(&expression).unwrap_err();
        assert_eq!(error.stage, Stage::Inference);
        assert_eq!(error.kind, ErrorKind::UnboundVariable);
    }
}

#[test]
#[ignore = "exercise: inference; parser不要"]
fn self_application_has_no_finite_type() {
    assert_eq!(
        infer(&fun("x", apply(var("x"), var("x"))))
            .unwrap_err()
            .kind,
        ErrorKind::InfiniteType
    );
}
