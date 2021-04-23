use std::path::{Path, PathBuf};

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
			filename, text, lines
		}
	}

	pub fn text(&self) -> &str { &self.text }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
}

pub trait FileSystem {
	type Root : ?Sized;

	fn get_file(&mut self, filename: &str) -> std::io::Result<File>;
}

pub struct GZDoomFolderFileSystem {
	pub root: PathBuf,
	pub nice_root_name: String
}

impl FileSystem for GZDoomFolderFileSystem {
	type Root = Path;

	fn get_file(&mut self, filename: &str) -> std::io::Result<File> {
		let p = self.root.join(Path::new(filename));
		let text = std::fs::read_to_string(p)?;
		Ok(File::new(
			format!("{}/{}", self.nice_root_name, filename),
			text
		))
    }
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_folder_filesystem() {
		let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		d.push("test_files");

		let mut sys = GZDoomFolderFileSystem { root: d, nice_root_name: "test_files".to_string() };
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
