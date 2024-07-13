#[derive(Default)]
pub struct SchemaData {
	pub views: Vec<SchemaView>
}

impl SchemaData {
	pub fn get_view(&mut self, name: String) -> Option<&mut SchemaView> {
		for view in &mut self.views {
			if view.name == name {
				return Some(view);
			}
		}
		
		None
	}
}

#[derive(Default)]
pub struct SchemaView {
	pub name: String,
	pub bones: Vec<SchemaBone>,
	pub images: Vec<SchemaImage>
}

#[derive(Default)]
pub struct SchemaBone {
	pub label: String,
	pub name: String,
	pub x: f32,
	pub y: f32
}

#[derive(Default, Clone)]
pub struct SchemaImage {
	pub file: String
}