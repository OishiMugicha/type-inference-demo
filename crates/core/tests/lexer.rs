use tiny_ml_core::{
    error::{ErrorKind, Stage},
    lex,
    span::Span,
    token::{Token, TokenKind as K},
};

#[test]
fn tokens_and_byte_spans() {
    assert_eq!(
        lex("let x = 12 in x + 3").unwrap(),
        vec![
            Token {
                kind: K::Let,
                span: Span::new(0, 3)
            },
            Token {
                kind: K::Ident("x".into()),
                span: Span::new(4, 5)
            },
            Token {
                kind: K::Equal,
                span: Span::new(6, 7)
            },
            Token {
                kind: K::Int(12),
                span: Span::new(8, 10)
            },
            Token {
                kind: K::In,
                span: Span::new(11, 13)
            },
            Token {
                kind: K::Ident("x".into()),
                span: Span::new(14, 15)
            },
            Token {
                kind: K::Plus,
                span: Span::new(16, 17)
            },
            Token {
                kind: K::Int(3),
                span: Span::new(18, 19)
            },
            Token {
                kind: K::Eof,
                span: Span::new(19, 19)
            },
        ]
    );
}

#[test]
fn keywords_operators_and_keyword_prefixes() {
    let kinds: Vec<_> =
        lex("if true then false else let in fun -> = + - * / < ( ) letter ifx _x x2")
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect();
    assert_eq!(
        kinds,
        vec![
            K::If,
            K::Bool(true),
            K::Then,
            K::Bool(false),
            K::Else,
            K::Let,
            K::In,
            K::Fun,
            K::Arrow,
            K::Equal,
            K::Plus,
            K::Minus,
            K::Star,
            K::Slash,
            K::Less,
            K::LParen,
            K::RParen,
            K::Ident("letter".into()),
            K::Ident("ifx".into()),
            K::Ident("_x".into()),
            K::Ident("x2".into()),
            K::Eof
        ]
    );
}

#[test]
fn empty_and_ascii_whitespace() {
    for input in ["", " \t\r\n"] {
        assert_eq!(
            lex(input).unwrap(),
            vec![Token {
                kind: K::Eof,
                span: Span::new(input.len(), input.len())
            }]
        );
    }
}

#[test]
fn invalid_character_has_utf8_byte_span() {
    let error = lex("1 + あ").unwrap_err();
    assert_eq!(error.stage, Stage::Lexer);
    assert_eq!(error.kind, ErrorKind::UnexpectedCharacter);
    assert_eq!(error.span, Some(Span::new(4, 7)));
}

#[test]
fn integer_boundaries_and_separate_minus() {
    assert_eq!(
        lex("9223372036854775807").unwrap()[0].kind,
        K::Int(i64::MAX)
    );
    assert_eq!(
        lex("9223372036854775808").unwrap_err().kind,
        ErrorKind::IntegerOutOfRange
    );
    let kinds: Vec<_> = lex("-42").unwrap().into_iter().map(|t| t.kind).collect();
    assert_eq!(kinds, vec![K::Minus, K::Int(42), K::Eof]);
}

#[test]
fn exact_whitespace_arrow_and_integer_spans() {
    let tokens = lex("fun _->00042").unwrap();
    assert_eq!(
        tokens[2],
        Token {
            kind: K::Arrow,
            span: Span::new(5, 7)
        }
    );
    assert_eq!(
        tokens[3],
        Token {
            kind: K::Int(42),
            span: Span::new(7, 12)
        }
    );
    for input in ["\u{b}", "\u{c}", "\u{a0}", "🙂"] {
        let error = lex(input).unwrap_err();
        assert_eq!(error.kind, ErrorKind::UnexpectedCharacter);
        assert_eq!(error.span, Some(Span::new(0, input.len())));
    }
    assert_eq!(
        lex(" 9223372036854775808").unwrap_err().span,
        Some(Span::new(1, 20))
    );
}
