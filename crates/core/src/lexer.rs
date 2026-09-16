use crate::{
    error::{Diagnostic, ErrorKind, LangResult, Stage},
    span::Span,
    token::{Token, TokenKind as K},
};

/// 空白を除くトークン列と、入力末尾のEofを返す。
pub fn lex(source: &str) -> LangResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        let mut end = start + ch.len_utf8();
        let kind = match ch {
            ' ' | '\t' | '\r' | '\n' => continue,
            '0'..='9' => {
                while let Some(&(offset, next)) = chars.peek() {
                    if !next.is_ascii_digit() {
                        break;
                    }
                    chars.next();
                    end = offset + 1;
                }
                K::Int(source[start..end].parse().map_err(|_| {
                    Diagnostic::new(
                        Stage::Lexer,
                        ErrorKind::IntegerOutOfRange,
                        "整数が許容範囲を超えています。",
                        Some(Span::new(start, end)),
                    )
                })?)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                while let Some(&(offset, next)) = chars.peek() {
                    if !next.is_ascii_alphanumeric() && next != '_' {
                        break;
                    }
                    chars.next();
                    end = offset + 1;
                }
                match &source[start..end] {
                    "true" => K::Bool(true),
                    "false" => K::Bool(false),
                    "if" => K::If,
                    "then" => K::Then,
                    "else" => K::Else,
                    "let" => K::Let,
                    "in" => K::In,
                    "fun" => K::Fun,
                    name => K::Ident(name.to_owned()),
                }
            }
            '-' => {
                if let Some(&(offset, '>')) = chars.peek() {
                    chars.next();
                    end = offset + 1;
                    K::Arrow
                } else {
                    K::Minus
                }
            }
            '=' => K::Equal,
            '+' => K::Plus,
            '*' => K::Star,
            '/' => K::Slash,
            '<' => K::Less,
            '(' => K::LParen,
            ')' => K::RParen,
            _ => {
                return Err(Diagnostic::new(
                    Stage::Lexer,
                    ErrorKind::UnexpectedCharacter,
                    format!("使えない文字です: {ch:?}"),
                    Some(Span::new(start, end)),
                ));
            }
        };
        tokens.push(Token {
            kind,
            span: Span::new(start, end),
        });
    }
    tokens.push(Token {
        kind: K::Eof,
        span: Span::new(source.len(), source.len()),
    });
    Ok(tokens)
}
