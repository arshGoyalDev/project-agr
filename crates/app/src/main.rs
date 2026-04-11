mod state;

use state::Browser;

fn main() -> iced::Result {
  iced::application("project-agr", Browser::update, Browser::view)
    .subscription(Browser::subscription)
    .theme(Browser::theme)
    .run_with(Browser::new)
}
