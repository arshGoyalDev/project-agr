pub mod loading;
pub mod navigation;
pub mod tabs;
pub mod window;

use crate::browser::Browser;
use crate::message::Message;
use iced::Task;

impl Browser {
  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      // Window & Canvas Events
      Message::TitleBarPressed => window::title_bar_pressed(),
      Message::MinimizeWindow => window::minimize_window(),
      Message::ToggleMaximizeWindow => window::toggle_maximize_window(),
      Message::CloseWindow => window::close_window(),
      Message::WindowResized(w, h) => window::window_resized(self, w, h),
      Message::ScrollChanged(offset) => window::scroll_changed(self, offset),
      Message::AddressInputChanged(text) => window::address_input_changed(self, text),
      Message::Click(x, y) => window::click(self, x, y),

      // Tab Management
      Message::TabHovered(index) => tabs::tab_hovered(self, index),
      Message::TabUnhovered => tabs::tab_unhovered(self),
      Message::CloseTab(index) => tabs::close_tab(self, index),
      Message::NewTab => tabs::new_tab(self),
      Message::SwitchTab(index) => tabs::switch_tab(self, index),

      // Navigation
      Message::NavigateTo(url) => navigation::navigate_to(self, url),
      Message::GoBack => navigation::go_back(self),
      Message::GoForward => navigation::go_forward(self),
      Message::LoadUrl(tab_index, url) => navigation::load_url(self, tab_index, url),

      // Background Loading Tasks
      Message::HtmlFetched(tab_index, base, is_view_source, res) => {
        loading::html_fetched(self, tab_index, base, is_view_source, res)
      }
      Message::CssFetched(tab_index, css_bodies) => {
        loading::css_fetched(self, tab_index, css_bodies)
      }
    }
  }
}
