pub mod loading;
pub mod navigation;
pub mod tabs;
pub mod window;

use iced::Task;

use crate::browser::Browser;
use crate::message::Message;

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
      Message::FontLoaded(_) => Task::none(),

      // Typing
      Message::KeyPressed(c) => window::key_pressed(self, c),
      Message::BackspacePressed => window::backspace_pressed(self),
      Message::BlinkCursor => window::blink_cursor(self),
      Message::EnterPressed => window::enter_pressed(self),
      Message::TabBlur => window::tab_blur(self),

      // Tab Management
      Message::TabHovered(index) => tabs::tab_hovered(self, index),
      Message::TabUnhovered => tabs::tab_unhovered(self),
      Message::CloseTab(index, curr_tab) => tabs::close_tab(self, index, curr_tab),
      Message::NewTab => tabs::new_tab(self),
      Message::SwitchTab(index) => tabs::switch_tab(self, index),

      // Navigation
      Message::ToggleBookmark => navigation::toggle_bookmark(self),
      Message::NavigateTo(url) => navigation::navigate_to(self, url),
      Message::GoBack => navigation::go_back(self),
      Message::GoForward => navigation::go_forward(self),
      Message::LoadUrl(tab_index, url, payload, reload, hard_reload) => {
        navigation::load_url(self, tab_index, url, payload, reload, hard_reload)
      }
      Message::Reload(tab_index, url, payload, hard_reload) => {
        navigation::reload(self, tab_index, url, payload, hard_reload)
      }
      Message::HtmlFetched(tab_index, base, is_view_source, res, reload, hard_reload) => {
        loading::html_fetched(
          self,
          tab_index,
          base,
          is_view_source,
          res,
          reload,
          hard_reload,
        )
      }
      Message::CssFetched(tab_index, css_bodies) => {
        loading::css_fetched(self, tab_index, css_bodies)
      }
      Message::ScrollToFragment(id) => navigation::scroll_to_fragment(self, id),
    }
  }
}
