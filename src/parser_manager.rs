#[cfg(feature = "multithreaded")]
use rayon::prelude::*;

use crate::ast::{TopLevel, TopLevelDefinitionKind};
use crate::err::ParsingError;
use crate::filesystem::{FileIndex, FileSystem, Files};
use crate::parser::Parser;

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug)]
pub struct FileSystemParseResult {
    pub asts: Vec<(FileIndex, TopLevel)>,
    pub errs: Vec<ParsingError>,
}

pub fn parse_filesystem<F: FileSystem>(
    mut filesystem: F,
    files: &mut Files,
) -> FileSystemParseResult {
    let mut ret = FileSystemParseResult {
        asts: vec![],
        errs: vec![],
    };
    let root_scripts = filesystem.get_files_no_ext("zscript");
    let mut needed_files = vec![];
    for r in root_scripts {
        let f = files.add(r);
        needed_files.push(f);
    }
    while !needed_files.is_empty() {
        #[cfg(feature = "multithreaded")]
        let iter = needed_files.par_iter();
        #[cfg(not(feature = "multithreaded"))]
        let iter = needed_files.iter();
        let res: Vec<_> = iter
            .map(|&f| Parser::new(f, files[f].text()).parse())
            .collect();

        needed_files.clear();
        for r in res {
            for d in r.ast.definitions.iter() {
                match &d.kind {
                    TopLevelDefinitionKind::Include(s) => {
                        let file = {
                            let filename = s.symbol.string();
                            filesystem.get_file(&filename).unwrap()
                        };
                        let f = files.add(file);
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
