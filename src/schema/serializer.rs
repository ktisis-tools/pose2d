use crate::schema::data::{SchemaBone, SchemaData, SchemaImage, SchemaView};
use std::{io::Read, path::PathBuf};
use quick_xml::{events::Event, errors::Error, Reader, Writer};

pub trait SchemaSerializer {
	fn serialize(&self) -> Result<Vec<u8>, Error>;
	fn deserialize(path: PathBuf) -> Self;
}

impl SchemaSerializer for SchemaData {
	fn serialize(&self) -> Result<Vec<u8>, Error> {
		let mut buffer = Vec::new();
		let mut writer = Writer::new_with_indent(&mut buffer, b'\t', 1);
		writer.write_bom()?;
		
		writer.create_element("Views")
			.write_inner_content(|inner| {
				for view in &self.views {
					// View
					inner.create_element("View")
						.with_attribute(("name", view.name.as_str()))
						.write_inner_content(|ele| {
							for img in &view.images {
								ele.create_element("Image")
									.with_attribute(("file", img.file.as_str()))
									.write_empty()?;
							}
							for bone in &view.bones {
								ele.create_element("Bone")
									.with_attribute(("label", bone.label.as_str()))
									.with_attribute(("name", bone.name.as_str()))
									.with_attribute(("x", bone.x.to_string().as_str()))
									.with_attribute(("y", bone.y.to_string().as_str()))
									.write_empty()?;
							}
							Ok::<(), Error>(())
						}).expect(&format!("Failed to write view: {}", view.name));
				}
				Ok::<(), Error>(())
			})?;
		
		Ok(buffer)
	}
	
	fn deserialize(path: PathBuf) -> Self {
		let mut reader = Reader::from_file(path)
			.expect("Failed to open reader.");
		
		let mut buf = Vec::new();
		
		let mut schema = SchemaData::default();
		let mut cur_view: Option<SchemaView> = None;
		
		loop {
			match reader.read_event_into(&mut buf) {
				Err(e) => panic!("Error as position {}: {:?}", reader.error_position(), e),
				Ok(Event::Eof) => break,
				Ok(Event::Start(e)) => {
					match e.name().as_ref() {
						b"View" => {
							let mut view = SchemaView::default();
							if let Ok(Some(attr)) = e.try_get_attribute("name") {
								attr.value.as_ref().read_to_string(&mut view.name).ok();
							}
							cur_view = Some(view);
						},
						_ => ()	
					};
				},
				Ok(Event::Empty(e)) => {
					match e.name().as_ref() {
						b"Image" => {
							let mut img = SchemaImage::default();
							if let Ok(Some(attr)) = e.try_get_attribute("file") {
								attr.value.as_ref().read_to_string(&mut img.file).ok();
							}
							cur_view.as_mut().unwrap().images.push(img);
						},
						b"Bone" => {
							let mut bone = SchemaBone::default();
							for attr in e.attributes() {
								if let Ok(attr) = attr {
									let mut name = String::new();
									let mut value = String::new();
									attr.key.as_ref().read_to_string(&mut name).ok();
									attr.value.as_ref().read_to_string(&mut value).ok();
									match name.as_str() {
										"label" => bone.label = value,
										"name" => bone.name = value,
										"x" => bone.x = value.parse().unwrap(),
										"y" => bone.y = value.parse().unwrap(),
										_ => ()
									}
								}
							}
							cur_view.as_mut().unwrap().bones.push(bone);
						},
						_ => ()
					};
				},
				Ok(Event::End(e)) => {
					match e.name().as_ref() {
						b"View" => {
							let view = cur_view.take().unwrap();
							schema.views.push(view);
						},
						_ => ()
					}
				}
				_ => ()
			};
		}
		
		schema
	}
}