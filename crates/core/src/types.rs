use std::{collections::BTreeMap, fmt};

pub type TypeVar = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Var(TypeVar),
    Function(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    pub quantified: Vec<TypeVar>,
    pub ty: Type,
}

pub type TypeEnv = BTreeMap<String, TypeScheme>;

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Var(id) => write!(f, "'t{id}"),
            Self::Function(input, output) => {
                if matches!(input.as_ref(), Self::Function(..)) {
                    write!(f, "({input}) -> {output}")
                } else {
                    write!(f, "{input} -> {output}")
                }
            }
        }
    }
}
