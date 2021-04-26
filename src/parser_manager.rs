use std::sync::{Arc, RwLock};

use rayon::prelude::*;

use crate::interner::{StringInterner, NameInterner, Interners};
use crate::err::ParsingError;
use crate::ast::{TopLevel, TopLevelDefinitionKind};
use crate::parser::Parser;
use crate::filesystem::{Files, FileSystem, FileIndex};

#[derive(Debug)]
pub struct FileSystemParseResult {
	pub files: Files,
	pub asts: Vec<(FileIndex, TopLevel)>,
	pub errs: Vec<ParsingError>
}

impl FileSystemParseResult {
	pub fn repr(&self, interners: &Interners) -> String {
		let mut ret = "".to_string();
		for e in self.errs.iter() {
			ret += &format!("{}\n", e.repr(&self.files));
		}

		for (f, ast) in self.asts.iter() {
			ret += &format!("{}:\n", self.files[*f].filename);
			ret += &ast.repr(interners);
			ret += "\n";
		}

		ret
	}
}

pub fn parse_filesystem<F: FileSystem>(
	mut filesystem: F, name_interner: Arc<RwLock<NameInterner>>, string_interner: Arc<RwLock<StringInterner>>
) -> FileSystemParseResult {
	let mut ret = FileSystemParseResult { files: Files::default(), asts: vec![], errs: vec![] };
	let root_scripts = filesystem.get_files_no_ext("zscript");
	let mut needed_files = vec![];
	for r in root_scripts {
		let f = ret.files.add(r);
		needed_files.push(f);
	}
	while !needed_files.is_empty() {
		let res: Vec<_> = needed_files.par_iter().map(|&f| {
			Parser::new(f, ret.files[f].text(), name_interner.clone(), string_interner.clone()).parse()
		}).collect();
		needed_files.clear();
		for r in res {
			for d in r.ast.definitions.iter() {
				match &d.kind {
					TopLevelDefinitionKind::Include(s) => {
						let file = {
							let l = string_interner.read().unwrap();
							let filename = l.string(s.symbol);
							filesystem.get_file(filename).unwrap()
						};
						let f = ret.files.add(file);
						needed_files.push(f);
					}
					_ => {}
				}
			}
			for e in r.errs {
				ret.errs.push(e);
			}
			ret.asts.push((r.file, r.ast));
		}
	}
	ret
}
