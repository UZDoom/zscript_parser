use vec1::Vec1;
use std::collections::BTreeMap;
use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use crate::Span;
use crate::filesystem::Files;
use crate::filesystem::FileIndex;

#[derive(Debug, Clone, PartialEq)]
pub struct FileSpans {
	pub file: FileIndex,
	pub main_spans: Vec1<Span>,
	pub info_spans: Vec<Span>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsingError {
	pub msg: String,
	pub file_spans: Vec<FileSpans>
}

impl ParsingError {
	fn get_lines(&self, files: &Files) -> ParsingErrorLines {
		let mut ret = ParsingErrorLines { file_lines: vec![] };
		for f in self.file_spans.iter() {
			let file = &files[f.file];
			let mut file_line = (f.file, BTreeMap::new());
			for s in f.main_spans.iter() {
				let span_lines = get_span_lines(&file.lines, *s);
				for l in span_lines.lines {
					let ranges =
						file_line.1
						.entry(l.line)
						.or_insert_with(|| ErrorLineRanges::default());

					ranges.main.push(l.range);
				}
			}
			for s in f.info_spans.iter() {
				let span_lines = get_span_lines(&file.lines, *s);
				for l in span_lines.lines {
					let ranges =
						file_line.1
						.entry(l.line)
						.or_insert_with(|| ErrorLineRanges::default());

					ranges.info.push(l.range);
				}
			}
			ret.file_lines.push(file_line);
		}
		ret
	}

	pub fn repr(&self, files: &Files) -> String {
		let mut ret = format!("error: {}\n", self.msg);
		let err_lines = self.get_lines(files);
		ret += &err_lines.repr(files);
		ret
	}
}

#[derive(Default)]
struct ErrorLineRanges {
	main: Vec<Range<usize>>,
	info: Vec<Range<usize>>
}

struct ParsingErrorLines {
	file_lines: Vec<(FileIndex, BTreeMap<usize, ErrorLineRanges>)>
}

impl ParsingErrorLines {
	fn repr(&self, files: &Files) -> String {
		let mut ret = "".to_string();
		for (file, lines_ranges) in self.file_lines.iter() {
			let file = &files[*file];
			let text = &*file.text;
			let max_line = *lines_ranges.keys().max().unwrap();
			let line_indicator_length = format!("{}", max_line + 1).len();
			let mut last_line = None;
			ret += &format!("{}--> {}\n", " ".repeat(line_indicator_length), file.filename);
			ret += &format!("{} |\n", " ".repeat(line_indicator_length));
			for (l, v) in lines_ranges.iter() {
				if let Some(last) = last_line {
					if *l == last + 2 {
						let last_line_range = &file.lines[l - 1];
						let last_line_text = &text[last_line_range.start..last_line_range.end];
						ret += &format!(
							"{:>width$} |{}\n", l - 1 + 1,
							last_line_text.replace("\t", "    "),
							width = line_indicator_length
						);
					} else if *l > last + 2 {
						ret += "...\n";
					}
				}
				last_line = Some(l);
				let line = &file.lines[*l];
				let line_text = &text[line.start..line.end];
				let mut p = vec![0u8; line_text.len() + 1];
				for r in v.info.iter() {
					let r = if r.start == r.end { r.start..(r.start + 1) } else { r.clone() };
					for x in r.clone() {
						p[x] = 1;
					}
				}
				for r in v.main.iter() {
					let r = if r.start == r.end { r.start..(r.start + 1) } else { r.clone() };
					for x in r.clone() {
						p[x] = 2;
					}
				}
				ret += &format!("{:>width$} |{}\n", l + 1, line_text.replace("\t", "    "), width = line_indicator_length);
				ret += &format!("{} |", " ".repeat(line_indicator_length));
				for (i, c) in line_text.char_indices() {
					ret += &match p[i] {
						2 => "^",
						1 => "-",
						_ => " "
					}.repeat(if c == '\t' { 4 } else { UnicodeWidthChar::width(c).unwrap_or(1) });
				}
				ret += match p[p.len() - 1] {
					2 => "^",
					1 => "-",
					_ => " "
				};
				ret += "\n";
			}
		}
		ret
	}
}

fn get_line(lines: &[Range<usize>], start: usize) -> usize {
	match lines.binary_search_by_key(&start, |Range { start, .. }| *start) {
		Ok(l)  => l,
		Err(l) => l - 1
	}
}

struct LineInfo {
	line: usize,
	range: Range<usize>
}

struct SpanLines {
	lines: Vec<LineInfo>
}

fn get_span_lines(lines: &[Range<usize>], span: Span) -> SpanLines {
	let mut line_index = get_line(lines, span.start);
	let mut ret = SpanLines { lines: vec![] };
	loop {
		let line = &lines[line_index];
		let col = if line.start > span.start { 0 } else { span.start - line.start };
		if span.end > line.end {
			let end_col = line.len();
			ret.lines.push(LineInfo {
				line: line_index,
				range: col..end_col
			});
			line_index += 1;
			continue;
		}
		let end_col = span.end - line.start;
		ret.lines.push(LineInfo {
			line: line_index,
			range: col..end_col
		});
		break;
	}
	ret
}
