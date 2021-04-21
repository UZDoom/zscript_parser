pub mod tokenizer;
pub mod parser;
pub mod ast;

pub mod interner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
	start: usize,
	end: usize
}

impl Span {
	#[inline(always)]
	fn combine(self, other: Span) -> Span {
		Span {
			start: self.start.min(other.start),
			end: self.end.max(other.end),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsingError {
	msg: String,
	spans: vec1::Vec1<Span>
}
