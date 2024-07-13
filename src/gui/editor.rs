use crate::{
	app::App,
	config::Configuration,
	gui::images::ImageLoader,
	schema::{
		SchemaFile,
		SchemaSerializer,
		data::{SchemaData, SchemaView, SchemaBone, SchemaImage}
	}
};
use std::{
	cell::RefCell,
	ops::DerefMut,
	path::PathBuf,
	rc::Rc
};
use easy_imgui::{ChildFlags, Color, Ui, Vector2, MouseButton, PopupFlags, TabItemFlags, Key};
use rfd::FileDialog;

// Editor

pub struct Editor {
	pub cfg: Configuration,
	images: Rc<RefCell<ImageLoader>>,
	file: Option<SchemaFile>,
	state: UiState
}

impl Editor {
	pub fn new(images: Rc<RefCell<ImageLoader>>) -> Self {
		Self {
			cfg: Configuration::open(),
			images,
			file: None,
			state: UiState::default()
		}
	}
	
	// UI Draw
	
	pub fn draw(&mut self, ui: &Ui<App>) {
		if self.file.is_none() {
			ui.text("No schema loaded.");
			return;
		}
		
		self.handle_keys(ui);
		
		ui.with_group(|| {
			self.draw_views(ui);
		});
		ui.same_line();
		ui.with_group(|| {
			if let Some(name) = &self.state.view {
				let active = name.clone();
				self.draw_view(ui, active);
			}
		});
	}
	
	fn draw_views(&mut self, ui: &Ui<App>) {
		let file = self.file.as_mut().unwrap();
		let data = &mut file.data;
		
		if ui.button("Add View") {
			let mut view = SchemaView::default();
			view.name = format!("View {}", data.views.len() + 1);
			self.state.open_view(&view);
			data.views.push(view);
		}
		
		ui.spacing();
		
		// View list
		
		let avail = ui.get_content_region_avail();
		ui.list_box_config("##views")
			.size(Vector2::new(avail.x * 0.15, avail.y))
			.with(|| {
				let mut i = 0;
				for view in &mut data.views {
					i += 1;
					
					let is_select = self.state.is_open_view(view);
					let is_click = ui.selectable_config(format!("{}##{}", view.name, i))
						.selected(is_select)
						.build();
					
					if is_click {
						self.state.open_view(view);
					}
					
					ui.popup_context_item_config()
						.str_id(format!("view_ctx_{i}"))
						.flags(PopupFlags::MouseButtonRight)
						.with(|| {
							ui.input_text_config("##name", &mut view.name).build();
							ui.set_keyboard_focus_here(0);
						});
				}
			});
	}
	
	fn draw_view(&mut self, ui: &Ui<App>, name: String) {
		let file = self.file.as_mut().unwrap();
		let data = &mut file.data;
		
		let view = data.get_view(name);
		if view.is_none() { return }
		
		let view = view.unwrap();
		
		// Bones
		
		ui.with_group(|| {
			if ui.button("Add Bone") {
				let mut bone = SchemaBone::default();
				bone.label = format!("Bone{}", view.bones.len());
				bone.name = format!("bone{}", view.bones.len());
				view.bones.push(bone);
			}

			ui.spacing();

			let avail = ui.get_content_region_avail();
			ui.list_box_config("##bones")
				.size(Vector2::new(avail.x * 0.15, avail.y))
				.with(|| {
					let mut i = 0;
					for bone in &mut view.bones {
						i += 1;

						let is_select = self.state.is_selected_bone(bone);
						let is_click = ui.selectable_config(format!("{}##{}", bone.label, i))
							.selected(is_select)
							.build();
						
						if is_click {
							self.state.select_bone(bone);
						}

						ui.popup_context_item_config()
							.str_id(format!("bone_ctx_{i}"))
							.flags(PopupFlags::MouseButtonRight)
							.with(|| {
								ui.input_text_config("##name", &mut bone.label).build();
								ui.set_keyboard_focus_here(0);
							});
					}
				});
		});
		
		ui.same_line();
		
		// Image tabs

		ui.with_group(|| {
			ui.tab_bar_config("##imgs").with(|| {
				ui.tab_item_config("+").flags(TabItemFlags::Trailing).with(|| {});
				if ui.is_item_clicked(MouseButton::Left) {
					let mut img = SchemaImage::default();
					img.file = format!("Image{}", view.images.len());
					self.state.open_tab = Some(img.file.clone());
					view.images.push(img);
				}
				
				let mut i = 0;
				let mut remove_at: Option<usize> = None;
				
				for img in &mut view.images {
					let file = &img.file.clone();
					let mut tab = ui.tab_item_config(format!("{}##{}", file, i));
					
					if let Some(open) = &self.state.open_tab {
						if img.file.eq(open) {
							tab = tab.flags(TabItemFlags::SetSelected);
							self.state.open_tab = None;
						}
					}
						
					tab.with(|| {
						let style = ui.style().get();

						ui.input_text_config("##name", &mut self.state.rename_img).build();
						if ui.is_item_deactivated_after_edit() {
							let name = self.state.rename_img.clone();
							img.file = name.clone();
							self.state.open_tab = Some(name);
						} else if !ui.is_item_activated() {
							self.state.rename_img = img.file.clone();
						}

						ui.same_line_ex(0.0, style.ItemInnerSpacing.x);

						ui.with_disabled(
							!(ui.is_key_down(Key::ModCtrl) && ui.is_key_down(Key::ModShift)),
							|| {
								if ui.button("X") {
									remove_at = Some(i);
								}
								
								ui.with_item_tooltip(|| {
									ui.text("Delete");
								});
							}
						);
						
						let mut images = self.images.borrow_mut();
						Self::draw_view_img(&mut self.state, ui, images.deref_mut(), &self.cfg, img);
						ui.same_line();
						Self::draw_view_bones(&mut self.state, ui, &mut view.bones);
					});
					
					i += 1;
				}
				
				if let Some(i) = remove_at {
					view.images.remove(i);
				}
			});
		});
	}
	
	fn draw_view_img(
		state: &mut UiState,
		ui: &Ui<App>,
		images: &mut ImageLoader,
		cfg: &Configuration,
		img: &mut SchemaImage
	) {
		if cfg.image_path.is_none() { return }
		
		let path = cfg.image_path.clone().unwrap();
		let path = PathBuf::from(path).join(&img.file);
		if !path.exists() {
			ui.text("File does not exist.");
			return;
		}
		
		let data = images.load(ui, path);
		if let Some(rect) = data.rect {
			let img = data.image.clone().unwrap();
			let scale = ui.get_content_region_avail().y / img.height() as f32;
			state.img_cursor = ui.get_cursor_screen_pos().into();
			state.img_size = [ img.width() as f32 * scale, img.height() as f32 * scale ];
			ui.image_with_custom_rect_config(rect, scale).build();
		}
	}
	
	fn draw_view_bones(
		state: &mut UiState,
		ui: &Ui<App>,
		bones: &mut Vec<SchemaBone>
	) {
		if state.bone.is_none() { return }
		
		if let Some(bone) = bones.iter_mut().find(|x| state.is_selected_bone(x)) {
			ui.child_config("##bone")
				.child_flags(ChildFlags::Border | ChildFlags::AutoResizeY)
				.with(|| {
					ui.text(&bone.label);
					ui.input_text_config("Bone", &mut bone.name).build();
					ui.drag_float_config("X", &mut bone.x).range(0.0, 1.0).speed(0.001).build();
					ui.drag_float_config("Y", &mut bone.y).range(0.0, 1.0).speed(0.001).build();
				});
		}
		
		const RADIUS: f32 = 10.0;
		
		let mouse_pos = ui.io().MousePos;
		
		let draw = ui.window_draw_list();
		for bone in bones {
			let pos = Vector2::from(state.img_cursor)
				+ Vector2::new(state.img_size[0] * bone.x, state.img_size[1] * bone.y);
			
			let hover = mouse_pos.x >= pos.x - RADIUS
				&& mouse_pos.x <= pos.x + RADIUS
				&& mouse_pos.y >= pos.y - RADIUS
				&& mouse_pos.y <= pos.y + RADIUS;
			
			if hover && ui.is_mouse_clicked(MouseButton::Left) {
				state.select_bone(bone);
			}
			
			let active = hover || state.is_selected_bone(bone);
			
			let color = if active { Color::WHITE } else { Color::new(1.0, 1.0, 1.0, 0.65) };
			let thick = if active { 2.5 } else { 1.5 };
			
			draw.add_circle_filled(pos, RADIUS, color, 32);
			draw.add_circle(pos, RADIUS, Color::BLACK, 32, thick);
		}
	}
	
	// Keys
	
	fn handle_keys(&mut self, ui: &Ui<App>) {
		if ui.is_key_down(Key::ModCtrl) && ui.is_key_pressed(Key::S) {
			self.save_file(false);
		}
	}
	
	// Data
	
	pub fn is_file_loaded(&self) -> bool {
		self.file.is_some()
	}
	
	pub fn open_file(&mut self) {
		let path = FileDialog::new()
			.add_filter("xml", &["xml"])
			.set_directory("/")
			.pick_file();
		
		if let Some(path) = path {
			let data = SchemaData::deserialize(path.clone());
			let file = SchemaFile {
				path: Some(path),
				data
			};
			self.file = Some(file);
		}
	}
	
	pub fn save_file(&mut self, new_path: bool) {
		if self.file.is_none() { return }

		println!("Saving...");

		let file = self.file.as_mut().unwrap();

		if file.path.is_none() || new_path {
			file.path = FileDialog::new()
				.add_filter("xml", &["xml"])
				.save_file();
		}

		if let Some(path) = &file.path {
			file.write(path);
		} else {
			println!("Save cancelled");
		}
	}

	pub fn new_file(&mut self) {
		println!("Creating new schema.");
		self.file = Some(SchemaFile::default());
	}
	
	pub fn save_config(&mut self) {
		self.cfg.save();
	}
}

// State

#[derive(Default)]
struct UiState {
	pub view: Option<String>,
	pub bone: Option<String>,
	pub open_tab: Option<String>,
	pub rename_img: String,
	pub img_cursor: [ f32; 2 ],
	pub img_size: [ f32; 2 ]
}

impl UiState {
	// View
	
	pub fn open_view(&mut self, view: &SchemaView) {
		self.view = Some(view.name.clone());
		self.bone = None;
	}
	
	pub fn is_open_view(&self, view: &SchemaView) -> bool {
		match &self.view {
			Some(name) => view.name.eq(name),
			None => false
		}
	}
	
	// Bone
	
	pub fn select_bone(&mut self, bone: &SchemaBone) {
		self.bone = Some(bone.label.clone());
	}
	
	pub fn is_selected_bone(&self, bone: &SchemaBone) -> bool {
		match &self.bone {
			Some(label) => bone.label.eq(label),
			None => false
		}
	}
}