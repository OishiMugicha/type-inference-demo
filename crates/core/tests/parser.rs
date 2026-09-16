use tiny_ml_core::{
    ast::{BinaryOp as Op, ExprKind as E},
    error::{ErrorKind, Stage},
    lex, parse,
    span::Span,
    token::{Token, TokenKind as K},
};

#[test]
fn literal_from_handwritten_tokens() {
    let tokens = vec![
        Token {
            kind: K::Int(42),
            span: Span::new(0, 2),
        },
        Token {
            kind: K::Eof,
            span: Span::new(2, 2),
        },
    ];
    let result = parse(&tokens).unwrap();
    assert_eq!(result.kind, E::Int(42));
    assert_eq!(result.span, Span::new(0, 2));
}

#[test]
fn malformed_token_stream_returns_error() {
    assert_eq!(parse(&[]).unwrap_err().kind, ErrorKind::UnexpectedToken);
    assert_eq!(
        parse(&[Token {
            kind: K::Int(1),
            span: Span::new(0, 1)
        }])
        .unwrap_err()
        .kind,
        ErrorKind::UnexpectedToken
    );
    let error = parse(&[Token {
        kind: K::Eof,
        span: Span::new(0, 0),
    }])
    .unwrap_err();
    assert_eq!(error.stage, Stage::Parser);
    assert_eq!(error.kind, ErrorKind::UnexpectedToken);
}

#[test]
fn multiplication_precedes_addition() {
    let result = parse(&lex("1 + 2 * 3").unwrap()).unwrap();
    assert_eq!(result.span, Span::new(0, 9));
    let E::Binary {
        op: Op::Add,
        left,
        right,
    } = result.kind
    else {
        panic!("expected addition")
    };
    assert_eq!(left.kind, E::Int(1));
    assert!(matches!(
        right.kind,
        E::Binary {
            op: Op::Multiply,
            ..
        }
    ));
}

#[test]
fn arithmetic_is_left_associative() {
    let result = parse(&lex("8 - 3 - 1").unwrap()).unwrap();
    let E::Binary {
        op: Op::Subtract,
        left,
        right,
    } = result.kind
    else {
        panic!("expected subtraction")
    };
    assert!(matches!(
        left.kind,
        E::Binary {
            op: Op::Subtract,
            ..
        }
    ));
    assert_eq!(right.kind, E::Int(1));
}

#[test]
fn application_is_left_associative_and_tight() {
    let result = parse(&lex("f x y + 1").unwrap()).unwrap();
    let E::Binary {
        op: Op::Add, left, ..
    } = result.kind
    else {
        panic!("expected addition")
    };
    let E::Apply { function, argument } = left.kind else {
        panic!("expected application")
    };
    assert!(matches!(function.kind, E::Apply { .. }));
    assert_eq!(argument.kind, E::Var("y".into()));
}

#[test]
fn parentheses_and_negation() {
    let result = parse(&lex("-(1 + 2)").unwrap()).unwrap();
    assert_eq!(result.span, Span::new(0, 8));
    let E::Negate(inner) = result.kind else {
        panic!("expected negation")
    };
    assert_eq!(inner.span, Span::new(1, 8));
    assert!(matches!(inner.kind, E::Binary { op: Op::Add, .. }));
    let sub = parse(&lex("f -1").unwrap()).unwrap();
    assert!(matches!(
        sub.kind,
        E::Binary {
            op: Op::Subtract,
            ..
        }
    ));
    let app = parse(&lex("f (-1)").unwrap()).unwrap();
    assert!(matches!(app.kind, E::Apply { .. }));
}

#[test]
fn let_function_and_conditional() {
    let result = parse(&lex("let f = fun x -> if x < 0 then 0 else x in f 2").unwrap()).unwrap();
    let E::Let { name, value, body } = result.kind else {
        panic!("expected let")
    };
    assert_eq!(name, "f");
    let E::Fun {
        parameter,
        body: fun_body,
    } = value.kind
    else {
        panic!("expected function")
    };
    assert_eq!(parameter, "x");
    assert!(matches!(fun_body.kind, E::If { .. }));
    assert!(matches!(body.kind, E::Apply { .. }));
}

#[test]
fn rejects_syntax_errors_and_trailing_input() {
    for input in [
        "1 < 2 < 3",
        "if true then 1",
        "let x = 1",
        "fun -> 1",
        "(1 + 2",
        "1 )",
        "1 in 2",
        "1 +",
    ] {
        let error = parse(&lex(input).unwrap()).expect_err(input);
        assert_eq!(error.stage, Stage::Parser, "{input}");
        assert_eq!(error.kind, ErrorKind::UnexpectedToken, "{input}");
    }
}

#[test]
fn negation_wraps_application_and_nested_forms_keep_spans() {
    let expr = parse(&lex(" -f x ").unwrap()).unwrap();
    assert_eq!(expr.span, Span::new(1, 5));
    let E::Negate(inner) = expr.kind else {
        panic!("expected negation")
    };
    assert!(matches!(inner.kind, E::Apply { .. }));
    let expr = parse(&lex("((true))").unwrap()).unwrap();
    assert_eq!(expr.span, Span::new(0, 8));
    assert_eq!(expr.kind, E::Bool(true));
    assert!(parse(&lex("let x = let y = 1 in y in if true then x else 0").unwrap()).is_ok());
}

#[test]
fn errors_point_to_unexpected_token_or_missing_input() {
    for (source, span) in [("1 )", Span::new(2, 3)), ("1 +", Span::new(3, 3))] {
        assert_eq!(parse(&lex(source).unwrap()).unwrap_err().span, Some(span));
    }
    let mut tokens = lex("1").unwrap();
    tokens.push(Token {
        kind: K::Int(2),
        span: Span::new(2, 3),
    });
    assert_eq!(parse(&tokens).unwrap_err().span, Some(Span::new(2, 3)));
}
