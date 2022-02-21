use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub(crate) filename: String,
    pub(crate) text: String,
    pub(crate) lines: Vec<std::ops::Range<usize>>,
}

impl File {
    pub fn new(filename: String, text: String) -> Self {
        let lines = get_lines(&text);
        Self {
            filename,
            text,
            lines,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileIndex(pub(crate) usize);

#[derive(Debug, Default)]
pub struct Files(Vec<File>);

impl std::ops::Index<FileIndex> for Files {
    type Output = File;

    fn index(&self, index: FileIndex) -> &Self::Output {
        &self.0[index.0]
    }
}

impl Files {
    pub fn add(&mut self, file: File) -> FileIndex {
        self.0.push(file);
        FileIndex(self.0.len() - 1)
    }

    pub fn text_from_span(&self, span: Span) -> &str {
        let file = &self[span.file];
        &file.text()[span.start..span.end]
    }
}

pub trait FileSystem {
    fn get_file(&mut self, filename: &str) -> Option<File>;
    fn get_files_no_ext(&mut self, filename: &str) -> Vec<File>;
}

#[cfg(feature = "gzd_folder_filesystem")]
pub use gzd_folder_filesystem::*;

#[cfg(feature = "gzd_folder_filesystem")]
mod gzd_folder_filesystem {
    use std::collections::HashMap;
    use std::path::Path;

    use walkdir::WalkDir;

    use super::*;

    pub struct GZDoomFolderFileSystem {
        files: HashMap<String, File>,
        files_no_ext: HashMap<String, Vec<File>>,
    }

    impl GZDoomFolderFileSystem {
        pub fn new(mut root: String, nice_root_name: String) -> std::io::Result<Self> {
            if !root.ends_with("/") {
                root += "/";
            }
            if !Path::new(&root).exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("root {:?} not found", root),
                ));
            }
            let subdirs = WalkDir::new(&root).follow_links(true);
            let mut files = HashMap::new();
            let mut files_no_ext = HashMap::new();
            for entry in subdirs {
                if entry.is_err() {
                    continue;
                }
                let p = entry.unwrap().into_path();
                if p == Path::new(".") || p == Path::new("..") {
                    continue;
                }
                if p.is_dir() {
                    continue;
                }
                let file_part = p.file_name().unwrap().to_string_lossy();
                if file_part.contains(".orig") {
                    continue;
                }
                if file_part.contains(".bak") {
                    continue;
                }
                if file_part.contains(".cache") {
                    continue;
                }

                let literal_filename = &p.to_string_lossy()[root.len()..].to_string();
                let filename = literal_filename.to_ascii_lowercase();
                let filename_no_ext =
                    p.with_extension("").to_string_lossy()[root.len()..].to_ascii_lowercase();
                let text = std::fs::read_to_string(p);
                if text.is_err() {
                    continue;
                }
                let text = text.unwrap();
                let f = File::new(format!("{}/{}", nice_root_name, literal_filename), text);
                files.insert(filename, f.clone());
                files_no_ext
                    .entry(filename_no_ext)
                    .or_insert_with(|| vec![])
                    .push(f);
            }
            Ok(Self {
                files,
                files_no_ext,
            })
        }
    }

    impl FileSystem for GZDoomFolderFileSystem {
        fn get_file(&mut self, filename: &str) -> Option<File> {
            self.files.get(&filename.to_ascii_lowercase()).cloned()
        }
        fn get_files_no_ext(&mut self, filename: &str) -> Vec<File> {
            self.files_no_ext
                .get(&filename.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| vec![])
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_folder_filesystem() {
            let d = format!("{}/test_files", env!("CARGO_MANIFEST_DIR"));

            let mut sys = GZDoomFolderFileSystem::new(d, "test_files".to_string());
            let f = sys.get_file("something.txt").unwrap();
            assert_eq!(
                f,
                File::new(
                    "test_files/something.txt".to_string(),
                    "Hello!\n".to_string()
                )
            );
        }
    }
}
