mod editor;
mod images;

use crate::{
	app::App,
	gui::{
		editor::Editor,
		images::ImageLoader
	}
};
use std::{cell::RefCell, rc::Rc};
use easy_imgui::{Cond, Ui, Key, Vector2, WindowFlags};

const VEC_ZERO: Vector2 = Vector2::new(0.0, 0.0);

pub struct Gui {
	pub images: Rc<RefCell<ImageLoader>>,
	editor: Editor,
	img_path_opening: bool
}

impl Gui {
	pub fn new() -> Self {
		let images = ImageLoader::new();
		let images_ref = Rc::new(RefCell::new(images));
		Self {
			images: images_ref.clone(),
			editor: Editor::new(images_ref),
			img_path_opening: false
		}
	}
	
	pub fn draw(&mut self, ui: &Ui<App>) {
		ui.set_next_window_pos(VEC_ZERO, Cond::Always, VEC_ZERO);
		ui.set_next_window_size(ui.display_size(), Cond::Always);
		ui.window_config("##main")
			.flags(WindowFlags::MenuBar | WindowFlags::NoDecoration)
			.with_always(|_| {
				self.draw_main(ui);
			});
		self.draw_popups(ui);
	}
	
	pub fn close(&mut self) {
		self.editor.save_config();
	}
	
	fn draw_main(&mut self, ui: &Ui<App>) {
		ui.with_always_menu_bar(|_| self.draw_menu_bar(ui));
		self.editor.draw(&ui);
	}
	
	fn draw_menu_bar(&mut self, ui: &Ui<App>) {
		ui.menu_config("File").with(|| {
			if ui.menu_item_config("New").build() {
				self.editor.new_file()
			}
			if ui.menu_item_config("Open").build() {
				self.editor.open_file();
			}
			
			ui.separator();
			
			let file_loaded = self.editor.is_file_loaded();
			if ui.menu_item_config("Save").enabled(file_loaded).build() {
				self.editor.save_file(false);
			}
			if ui.menu_item_config("Save As").enabled(file_loaded).build() {
				self.editor.save_file(true);
			}
		});
		
		ui.menu_config("Options").with(|| {
			if ui.menu_item_config("Set image path...").build() {
				self.img_path_opening = true;
			}
		});
	}
	
	fn draw_popups(&mut self, ui: &Ui<App>) {
		const POPUP_NAME: &str = "Set image path";
		
		if self.img_path_opening {
			ui.open_popup(POPUP_NAME);
			self.img_path_opening = false;
		}
		
		ui.popup_modal_config(POPUP_NAME)
			.close_button(true)
			.flags(WindowFlags::AlwaysAutoResize)
			.with(|| {
				let cfg = &mut self.editor.cfg;

				let mut is_default = false;
				let mut default = "".to_string();
				let text = match &mut cfg.image_path {
					Some(path) => path,
					_ => {
						is_default = true;
						&mut default
					}
				};
				
				if ui.input_text_config("##path", text).build() && is_default {
					cfg.image_path = Some(text.to_string());
				}
				
				if ui.is_key_pressed(Key::Enter) {
					ui.close_current_popup();
					self.editor.cfg.save();
				}
			});
	}
}