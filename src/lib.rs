pub mod parser;
pub mod tokenizer;

pub mod ast;
pub mod hir;
pub mod ir_common;

pub mod err;
pub mod filesystem;
pub mod interner;
pub mod parser_manager;

#[cfg(feature = "serialize")]
use serde::Serialize;

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    file: filesystem::FileIndex,
    start: usize,
    end: usize,
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
            file: self.file,
        }
    }
}

fn get_lines(source: &str) -> Vec<std::ops::Range<usize>> {
    let source_ptr = source.as_ptr() as usize;
    source
        .split('\n')
        .map(|l| {
            let l_ptr = l.as_ptr() as usize;
            let offset = l_ptr - source_ptr;
            offset..(offset + l.len())
        })
        .collect()
}
