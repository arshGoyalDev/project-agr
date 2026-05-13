mod browser;
mod canvas;
mod dom;
mod message;
mod net;
mod tab;
mod update;
mod view;
mod window_controls;

use browser::Browser;

fn main() -> iced::Result {
  iced::application("project-agr", Browser::update, Browser::view)
    .subscription(Browser::subscription)
    .theme(Browser::theme)
    .window(iced::window::Settings {
      decorations: false,
      ..Default::default()
    })
    .run_with(Browser::new)
}
