pub mod tokenizer;
pub mod parser;
pub mod ast;

pub mod interner;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsingError {
	msg: String
}
