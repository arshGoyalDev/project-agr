mod state;
mod tab;
mod window_controls;

use iced::window;
use state::Browser;

fn main() -> iced::Result {
  iced::application("project-agr", Browser::update, Browser::view)
    .subscription(Browser::subscription)
    .theme(Browser::theme)
    .window(window::Settings {
      decorations: false,
      ..Default::default()
    })
    .run_with(Browser::new)
}
