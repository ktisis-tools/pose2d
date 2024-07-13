use crate::schema::{data::SchemaData, SchemaSerializer};
use std::{io::Write, fs::OpenOptions, path::PathBuf};

#[derive(Default)]
pub struct SchemaFile {
	pub path: Option<PathBuf>,
	pub data: SchemaData
}

impl SchemaFile {
	pub fn write(&self, path: &PathBuf) {
		let mut file = OpenOptions::new()
			.create(true)
			.write(true)
			.open(path)
			.expect("Failed to open file.");
		
		let buffer = self.data.serialize()
			.expect("Failed to serialize file.");
		
		file.write_all(&buffer)
			.and(file.set_len(buffer.len() as u64))
			.expect("Failed to write file.");
	}
}