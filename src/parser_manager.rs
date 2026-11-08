#[cfg(feature = "multithreaded")]
use rayon::prelude::*;

use crate::ast::{TopLevel, TopLevelDefinitionKind};
use crate::err::{ParsingError, ParsingErrorLevel};
use crate::filesystem::{FileIndex, FileSystem, Files};
use crate::parser::Parser;
use vec1::vec1;

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug)]
pub struct FileIndexAndAst {
    pub file: FileIndex,
    pub ast: TopLevel,
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug)]
pub struct FileSystemParseResult {
    pub asts: Vec<FileIndexAndAst>,
}

pub struct ParseFileSystemConfig<'a> {
    pub root_name: &'a str,
}
impl<'a> Default for ParseFileSystemConfig<'a> {
    fn default() -> Self {
        Self {
            root_name: "zscript",
        }
    }
}

pub fn parse_filesystem<F: FileSystem>(
    filesystem: F,
    files: &mut Files,
    errs: &mut Vec<ParsingError>,
) -> FileSystemParseResult {
    parse_filesystem_config(filesystem, files, errs, &ParseFileSystemConfig::default())
}

fn resolve_relative_include_path(from_file: &str, include: &str) -> Result<String, String> {
    if !include.starts_with("./") && !include.starts_with("../") {
        return Ok(include.to_string());
    }
    let mut adjusted_include = include;
    let mut ret = from_file
        .rsplit_once("/")
        .map(|(left, _right)| left)
        .unwrap_or("")
        .to_string();
    if let Some(a) = adjusted_include.strip_prefix("./") {
        adjusted_include = a;
    }
    while let Some(a) = adjusted_include.strip_prefix("../") {
        adjusted_include = a;
        let Some(left) = ret.rsplit_once("/").map(|(left, _right)| left) else {
            return Err(format!(
                "too many `../`s for path relative to `{from_file}`"
            ));
        };
        ret = left.to_string();
    }
    ret += "/";
    ret += adjusted_include;
    Ok(ret)
}

pub fn parse_filesystem_config<F: FileSystem>(
    mut filesystem: F,
    files: &mut Files,
    errs: &mut Vec<ParsingError>,
    config: &ParseFileSystemConfig,
) -> FileSystemParseResult {
    let mut ret = FileSystemParseResult { asts: vec![] };
    let root_scripts = filesystem.get_files_no_ext(config.root_name);
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
                if let TopLevelDefinitionKind::Include(s) = &d.kind {
                    let file = {
                        let maybe_filename = resolve_relative_include_path(
                            &files[r.file].filename,
                            &s.symbol.string(),
                        );
                        let filename = match maybe_filename {
                            Ok(filename) => filename,
                            Err(e) => {
                                errs.push(ParsingError {
                                    level: ParsingErrorLevel::Error,
                                    msg: e,
                                    main_spans: vec1![s.span],
                                    info_spans: vec![],
                                });
                                continue;
                            }
                        };
                        let maybe_file = filesystem.get_file(&filename);
                        let Some(f) = maybe_file else {
                            errs.push(ParsingError {
                                level: ParsingErrorLevel::Error,
                                msg: format!("couldn't find file in #include statement (full path: {filename})"),
                                main_spans: vec1![s.span],
                                info_spans: vec![],
                            });
                            continue;
                        };
                        f
                    };
                    let f = files.add(file);
                    needed_files.push(f);
                }
            }
            for e in r.errs {
                errs.push(e);
            }
            ret.asts.push(FileIndexAndAst {
                file: r.file,
                ast: r.ast,
            });
        }
    }
    ret
}
