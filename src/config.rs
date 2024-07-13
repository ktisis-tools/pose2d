use std::{
	env::current_exe,
	fs::{read_to_string, OpenOptions},
	io::Write,
	path::PathBuf
};
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Configuration {
	pub image_path: Option<String>
}

impl Configuration {
	pub fn open() -> Self {
		Self::read()
	}
	
	pub fn save(&self) {
		let path = Self::get_path().unwrap();
		let mut file = OpenOptions::new()
			.create(true)
			.write(true)
			.open(path)
			.unwrap();
		
		let content = serde_json::to_string(&self).unwrap();
		file.write_all(content.as_bytes()).expect("Failed to write config.");
		file.set_len(content.len() as _).unwrap();
	}
	
	fn read() -> Configuration {
		let path = Self::get_path().unwrap();
		if path.exists() {
			let content = read_to_string(path).expect("Failed to read config file.");
			serde_json::from_str(&content).expect("Failed to parse config.")
		} else {
			println!("No configuration found.");
			Configuration::default()
		}
	}
	
	fn get_path() -> Option<PathBuf> {
		let dir = current_exe();
		match &dir {
			Ok(path) => {
				match path.parent() {
					Some(path) => Some(path.join("data.json")),
					_ => None
				}
			},
			_ => None
		}
	}
}