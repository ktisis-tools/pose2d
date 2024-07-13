use crate::app::App;
use std::{
	fs,
	collections::HashMap,
	path::PathBuf
};
use easy_imgui::{
	CustomRectIndex, Ui, FontAtlasMut,
	image::{load_from_memory_with_format, ImageFormat, DynamicImage, GenericImage}
};

// ImageLoader

pub struct ImageLoader {
	images: HashMap<PathBuf, ImageData>
}

impl ImageLoader {
	pub fn new() -> ImageLoader {
		ImageLoader {
			images: HashMap::new()
		}
	}
	
	pub fn build_atlas(&mut self, atlas: &mut FontAtlasMut<'_, App>) {
		println!("Building atlas.");
		
		for (path, data) in &mut self.images {
			if let Some(img) = &data.image {
				println!("Loading {}", path.display());
				
				let img = img.clone();
				let rect = atlas.add_custom_rect_regular([img.width(), img.height()], move |_, pixels| {
					pixels.copy_from(&img, 0, 0).unwrap();
				});
				data.rect = Some(rect);
			}
		}
	}
	
	pub fn update(&mut self) {
		let mut remove_paths = Vec::<PathBuf>::new();
		
		for (path, img) in &mut self.images {
			if img.used {
				img.used = false;
			} else {
				//println!("culling {}", path.display());
				remove_paths.push(path.clone());
			}
		}
	}
	
	pub fn load(&mut self, ui: &Ui<App>, path: PathBuf) -> &ImageData {
		 if !self.images.contains_key(&path) {
			let mut img: Option<DynamicImage> = None;

			if path.exists() {
				if let Ok(bytes) = fs::read(&path) {
					let result = load_from_memory_with_format(&bytes, ImageFormat::Png);
					if let Ok(value) = result {
						img = Some(value);
					} else if let Err(err) = result {
						println!("{}", err);
					}
				}
			}
			
			self.images.insert(path.clone(), ImageData::new(img));
			ui.invalidate_font_atlas(); 
		}
		
		let data = self.images.get_mut(&path).unwrap();
		data.used = true;
		return data;
	}
}

// ImageData

pub struct ImageData {
	pub used: bool,
	pub image: Option<DynamicImage>,
	pub rect: Option<CustomRectIndex>
}

impl ImageData {
	pub fn new(image: Option<DynamicImage>) -> Self {
		Self {
			used: true,
			image,
			rect: None
		}
	}
}