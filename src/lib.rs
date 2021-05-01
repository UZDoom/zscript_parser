pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod err;
pub mod filesystem;
pub mod parser_manager;
pub mod interner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
	start: usize,
	end: usize,
	file: filesystem::FileIndex
}

impl Span {
	#[inline(always)]
	fn combine(self, other: Span) -> Span {
		// note - this function expects the spans to come
		// from the same file
		// it doesn't actually check this though
		Span {
			start: self.start.min(other.start),
			end: self.end.max(other.end),
			file: self.file
		}
	}
}

fn get_lines(source: &str) -> Vec<std::ops::Range<usize>> {
	let source_ptr = source.as_ptr() as usize;
	source.split('\n').map(|l| {
		let l_ptr = l.as_ptr() as usize;
		let offset = l_ptr - source_ptr;
		offset..(offset + l.len())
	}).collect()
}
