use std::{collections::BTreeMap, fmt, rc::Rc};

use crate::ast::Expr;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Closure(Rc<Closure>),
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub parameter: String,
    pub body: Expr,
    pub environment: ValueEnv,
}

pub type ValueEnv = BTreeMap<String, Value>;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Closure(_) => write!(f, "<fun>"),
        }
    }
}
