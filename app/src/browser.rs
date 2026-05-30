use crate::message::Message;
use crate::tab::Tab;
use iced::{Subscription, Task, font, time, window};
use std::env;

pub struct Browser {
  pub tabs: Vec<Tab>,
  pub active_tab_index: usize,
  pub hovered_tab: Option<usize>,
  pub address_bar_text: String,
  pub width: f32,
  pub height: f32,
  pub bookmarks: Vec<String>,
  pub cursor_blink_visible: bool,
  pub pending_resubmit_index: Option<usize>,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("about:blank");
    if let Some(value) = env::args().nth(1) {
      url = value;
    }

    let initial_tab = Tab::new(url.clone());

    let load_url_task = Task::done(Message::LoadUrl(0, url.clone(), None, false, false));

    let load_font_task =
      font::load(include_bytes!("../../assets/bootstrap-icons.ttf").as_slice())
        .map(Message::FontLoaded);

    (
      Self {
        tabs: vec![initial_tab],
        active_tab_index: 0,
        hovered_tab: None,
        address_bar_text: url.clone(),
        width: 800.0,
        height: 600.0,
        bookmarks: Vec::new(),
        cursor_blink_visible: true,
        pending_resubmit_index: None,
      },
      Task::batch(vec![load_font_task, load_url_task]),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    Subscription::batch(vec![
      window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height)),
      time::every(std::time::Duration::from_millis(530)).map(|_| Message::BlinkCursor),
    ])
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
