#![allow(dead_code)]
use std::collections::BTreeMap;
use tiny_ml_core::{
    ast::{BinaryOp, Expr, ExprKind},
    span::Span,
    types::{Type, TypeVar},
};

// テスト用ASTの位置はダミー。parserの位置情報はparser.rsで検証する。
pub fn expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::default())
}
pub fn int(value: i64) -> Expr {
    expr(ExprKind::Int(value))
}
pub fn boolean(value: bool) -> Expr {
    expr(ExprKind::Bool(value))
}
pub fn var(name: &str) -> Expr {
    expr(ExprKind::Var(name.into()))
}
pub fn fun(name: &str, body: Expr) -> Expr {
    expr(ExprKind::Fun {
        parameter: name.into(),
        body: Box::new(body),
    })
}
pub fn apply(function: Expr, argument: Expr) -> Expr {
    expr(ExprKind::Apply {
        function: Box::new(function),
        argument: Box::new(argument),
    })
}
pub fn bind(name: &str, value: Expr, body: Expr) -> Expr {
    expr(ExprKind::Let {
        name: name.into(),
        value: Box::new(value),
        body: Box::new(body),
    })
}
pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    expr(ExprKind::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}
pub fn branch(condition: Expr, yes: Expr, no: Expr) -> Expr {
    expr(ExprKind::If {
        condition: Box::new(condition),
        then_branch: Box::new(yes),
        else_branch: Box::new(no),
    })
}
pub fn negate(value: Expr) -> Expr {
    expr(ExprKind::Negate(Box::new(value)))
}
pub fn arrow(input: Type, output: Type) -> Type {
    Type::Function(Box::new(input), Box::new(output))
}

/// テストの比較用。型変数番号の全単射による名前の違いだけを無視する。
/// 単一化や型推論には使わない。
pub fn assert_type_eq(actual: Type, expected: Type) {
    fn same(
        a: &Type,
        b: &Type,
        ab: &mut BTreeMap<TypeVar, TypeVar>,
        ba: &mut BTreeMap<TypeVar, TypeVar>,
    ) -> bool {
        match (a, b) {
            (Type::Int, Type::Int) | (Type::Bool, Type::Bool) => true,
            (Type::Var(x), Type::Var(y)) => {
                *ab.entry(*x).or_insert(*y) == *y && *ba.entry(*y).or_insert(*x) == *x
            }
            (Type::Function(a1, a2), Type::Function(b1, b2)) => {
                same(a1, b1, ab, ba) && same(a2, b2, ab, ba)
            }
            _ => false,
        }
    }
    assert!(
        same(
            &actual,
            &expected,
            &mut BTreeMap::new(),
            &mut BTreeMap::new()
        ),
        "actual: {actual}, expected: {expected}"
    );
}
