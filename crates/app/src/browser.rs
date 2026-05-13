use crate::message::Message;
use crate::tab::Tab;
use iced::{Subscription, Task, window};
use std::env;

pub struct Browser {
  pub tabs: Vec<Tab>,
  pub active_tab_index: usize,
  pub hovered_tab: Option<usize>,
  pub address_bar_text: String,
  pub width: f32,
  pub height: f32,
  pub bookmarks: Vec<String>,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("about:blank");
    if let Some(value) = env::args().nth(1) {
      url = value;
    }

    let initial_tab = Tab::new(url.clone());

    (
      Self {
        tabs: vec![initial_tab],
        active_tab_index: 0,
        hovered_tab: None,
        address_bar_text: url.clone(),
        width: 800.0,
        height: 600.0,
        bookmarks: Vec::new(),
      },
      Task::done(Message::LoadUrl(0, url)),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height))
  }

  pub fn relayout(&mut self) {
    let width = self.width;
    let tab = &mut self.tabs[self.active_tab_index];

    if let Some(doc) = &mut tab.document {
      doc.layout(width);
      tab.display_list = doc.paint();
      tab.max_y = tab.display_list.max_y();
    }
  }

  pub fn theme(&self) -> iced::Theme {
    iced::Theme::Dark
  }
}
