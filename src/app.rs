use crate::gui::Gui;
use easy_imgui::{Ui, UiBuilder, FontAtlasMut};
use easy_imgui_window::{
	winit,
	winit::event_loop::EventLoop,
	Application, AppHandler, Args, EventResult
};
use winit::event::WindowEvent;

pub struct App {
	gui: Gui
}

impl App {
	pub fn run() {
		let event_loop = EventLoop::new().unwrap();
		let mut main = AppHandler::<App>::default();
		main.attributes().title = String::from("Pose2D Editor");
		event_loop.run_app(&mut main).expect("Event loop failure");
	}
}

impl Application for App {
	type UserEvent = ();
	type Data = ();

	fn new(_args: Args<'_, Self::Data>) -> Self {
		Self {
			gui: Gui::new()
		}
	}

	fn window_event(&mut self, args: Args<'_, Self::Data>, _event: WindowEvent, res: EventResult) {
		if res.window_closed {
			self.gui.close();
			args.event_loop.exit();
		}
	}
}

impl UiBuilder for App {
	fn build_custom_atlas(&mut self, atlas: &mut FontAtlasMut<'_, Self>) {
		self.gui.images.borrow_mut().build_atlas(atlas);
	}
	
	fn do_ui(&mut self, ui: &Ui<Self>) {
		self.gui.draw(ui);
		self.gui.images.borrow_mut().update();
	}
}