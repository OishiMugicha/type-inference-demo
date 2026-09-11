use tiny_ml_core::{
    ast::{BinaryOp as Op, ExprKind as E},
    error::{ErrorKind, Stage},
    lex, parse,
    span::Span,
    token::{Token, TokenKind as K},
};

#[test]
#[ignore = "exercise: parser; lexer不要"]
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
#[ignore = "exercise: parser; lexer不要"]
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
#[ignore = "exercise: lexer + parser"]
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
#[ignore = "exercise: lexer + parser"]
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
#[ignore = "exercise: lexer + parser"]
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
#[ignore = "exercise: lexer + parser"]
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
#[ignore = "exercise: lexer + parser"]
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
#[ignore = "exercise: lexer + parser"]
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
