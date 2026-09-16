use crate::{
    ast::{BinaryOp, Expr, ExprKind as E},
    error::{Diagnostic, ErrorKind, LangResult, Stage},
    span::Span,
    token::{Token, TokenKind as K},
};

/// Eofまでを消費して、単一の式を返す。
pub fn parse(tokens: &[Token]) -> LangResult<Expr> {
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let expr = parser.expression()?;
    parser.expect(&K::Eof)?;
    if parser.position != tokens.len() {
        return Err(parser.error("入力末尾"));
    }
    Ok(expr)
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&K> {
        self.tokens.get(self.position).map(|token| &token.kind)
    }

    fn error(&self, expected: &str) -> Diagnostic {
        let span = self
            .tokens
            .get(self.position)
            .map(|token| token.span)
            .unwrap_or_else(|| {
                let end = self.tokens.last().map_or(0, |token| token.span.end);
                Span::new(end, end)
            });
        Diagnostic::new(
            Stage::Parser,
            ErrorKind::UnexpectedToken,
            format!("{expected}が必要です。見つかったもの: {:?}", self.peek()),
            Some(span),
        )
    }

    fn expect(&mut self, kind: &K) -> LangResult<Span> {
        if self.peek() != Some(kind) {
            return Err(self.error(&format!("{kind:?}")));
        }
        let span = self.tokens[self.position].span;
        self.position += 1;
        Ok(span)
    }

    fn identifier(&mut self) -> LangResult<String> {
        let Some(K::Ident(name)) = self.peek() else {
            return Err(self.error("識別子"));
        };
        let name = name.clone();
        self.position += 1;
        Ok(name)
    }

    fn expression(&mut self) -> LangResult<Expr> {
        match self.peek() {
            Some(K::Let) => {
                let start = self.expect(&K::Let)?.start;
                let name = self.identifier()?;
                self.expect(&K::Equal)?;
                let value = Box::new(self.expression()?);
                self.expect(&K::In)?;
                let body = Box::new(self.expression()?);
                let span = Span::new(start, body.span.end);
                Ok(Expr::new(E::Let { name, value, body }, span))
            }
            Some(K::If) => {
                let start = self.expect(&K::If)?.start;
                let condition = Box::new(self.expression()?);
                self.expect(&K::Then)?;
                let then_branch = Box::new(self.expression()?);
                self.expect(&K::Else)?;
                let else_branch = Box::new(self.expression()?);
                let span = Span::new(start, else_branch.span.end);
                Ok(Expr::new(
                    E::If {
                        condition,
                        then_branch,
                        else_branch,
                    },
                    span,
                ))
            }
            Some(K::Fun) => {
                let start = self.expect(&K::Fun)?.start;
                let parameter = self.identifier()?;
                self.expect(&K::Arrow)?;
                let body = Box::new(self.expression()?);
                let span = Span::new(start, body.span.end);
                Ok(Expr::new(E::Fun { parameter, body }, span))
            }
            _ => self.comparison(),
        }
    }

    fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        let span = Span::new(left.span.start, right.span.end);
        Expr::new(
            E::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        )
    }

    fn comparison(&mut self) -> LangResult<Expr> {
        let left = self.additive()?;
        if self.peek() == Some(&K::Less) {
            self.position += 1;
            Ok(Self::binary(BinaryOp::Less, left, self.additive()?))
        } else {
            Ok(left)
        }
    }

    fn additive(&mut self) -> LangResult<Expr> {
        let mut expr = self.product()?;
        loop {
            let op = match self.peek() {
                Some(K::Plus) => BinaryOp::Add,
                Some(K::Minus) => BinaryOp::Subtract,
                _ => break,
            };
            self.position += 1;
            expr = Self::binary(op, expr, self.product()?);
        }
        Ok(expr)
    }

    fn product(&mut self) -> LangResult<Expr> {
        let mut expr = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(K::Star) => BinaryOp::Multiply,
                Some(K::Slash) => BinaryOp::Divide,
                _ => break,
            };
            self.position += 1;
            expr = Self::binary(op, expr, self.unary()?);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> LangResult<Expr> {
        if self.peek() == Some(&K::Minus) {
            let start = self.expect(&K::Minus)?.start;
            let inner = self.unary()?;
            let span = Span::new(start, inner.span.end);
            Ok(Expr::new(E::Negate(Box::new(inner)), span))
        } else {
            self.application()
        }
    }

    fn application(&mut self) -> LangResult<Expr> {
        let mut expr = self.atom()?;
        while matches!(
            self.peek(),
            Some(K::Int(_) | K::Bool(_) | K::Ident(_) | K::LParen)
        ) {
            let argument = self.atom()?;
            let span = Span::new(expr.span.start, argument.span.end);
            expr = Expr::new(
                E::Apply {
                    function: Box::new(expr),
                    argument: Box::new(argument),
                },
                span,
            );
        }
        Ok(expr)
    }

    fn atom(&mut self) -> LangResult<Expr> {
        let Some(token) = self.tokens.get(self.position) else {
            return Err(self.error("式"));
        };
        let kind = match &token.kind {
            K::Int(value) => E::Int(*value),
            K::Bool(value) => E::Bool(*value),
            K::Ident(name) => E::Var(name.clone()),
            K::LParen => {
                let start = self.expect(&K::LParen)?.start;
                let mut expr = self.expression()?;
                let end = self.expect(&K::RParen)?.end;
                expr.span = Span::new(start, end);
                return Ok(expr);
            }
            _ => return Err(self.error("整数・真偽値・識別子・括弧付きの式")),
        };
        let span = token.span;
        self.position += 1;
        Ok(Expr::new(kind, span))
    }
}
