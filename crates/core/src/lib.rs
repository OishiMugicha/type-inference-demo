//! 小さなML言語の練習用API。仕様は docs/language.md を参照。
pub mod ast;
pub mod error;
pub mod eval;
pub mod infer;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;
pub mod types;
pub mod value;

pub use eval::eval;
pub use infer::infer;
pub use lexer::lex;
pub use parser::parse;
