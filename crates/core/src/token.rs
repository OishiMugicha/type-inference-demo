use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Int(i64),
    Bool(bool),
    Ident(String),
    If,
    Then,
    Else,
    Let,
    In,
    Fun,
    Arrow,
    Equal,
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
