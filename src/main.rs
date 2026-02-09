mod app;
mod net;
mod rendering;
mod ui;
mod utils;

use app::Browser;

fn main() -> iced::Result {
  iced::application("project-agr", Browser::update, Browser::view)
    .subscription(Browser::subscription)
    .run_with(Browser::new)
}