use iced::widget::{canvas, container};
use iced::{Element, Subscription, Task, window};

use html_parser::{HTMLParser, Node};
use layout::{DisplayList, DocumentLayout, paint_tree_document, syntax_highlight};
use net::URLHandler;
use ui::{BrowserCanvas, Message};

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

pub struct Browser {
  pub display_list: DisplayList,
  pub scroll_offset: f32,
  pub current_url: String,
  pub max_y: f32,
  pub width: f32,
  pub height: f32,
  pub tree: Option<Rc<RefCell<Node>>>,
  pub document: Option<DocumentLayout>,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("about:blank");
    let args: Vec<String> = env::args().collect();

    if let Some(value) = args.get(1) {
      url = value.to_string();
    }

    (
      Self {
        display_list: DisplayList::new(),
        scroll_offset: 0.0,
        max_y: 0.0,
        current_url: url,
        tree: None,
        document: None,
        width: 0.0,
        height: 0.0,
      },
      Task::done(Message::LoadUrl()),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height))
  }

  fn relayout(&mut self) {
    if let Some(doc) = &mut self.document {
      doc.layout(self.width);

      self.display_list = DisplayList::new();
      paint_tree_document(doc, &mut self.display_list);

      self.max_y = self.display_list.max_y();
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::ScrollChanged(offset) => {
        self.scroll_offset = offset;
        Task::none()
      }

      Message::LoadUrl() => {
        let mut url_handler = URLHandler::default();
        url_handler.init(self.current_url.clone(), false);

        let body_result = url_handler.request();

        match body_result {
          Ok(value) => {
            let mut html_parser = HTMLParser::new(value);
            self.tree = Some(html_parser.parse());
          }
          _ => (),
        }

        // Handle view-source syntax highlighting
        if url_handler.view_source {
          if let Some(node) = &self.tree {
            let highlighted = syntax_highlight(node);
            let mut html_parser = HTMLParser::new(highlighted);
            self.tree = Some(html_parser.parse());
          }
        }

        // Build DocumentLayout and paint
        if let Some(node) = &self.tree {
          let mut doc = DocumentLayout::new(node);
          doc.layout(self.width);

          self.display_list = DisplayList::new();
          paint_tree_document(&doc, &mut self.display_list);

          self.max_y = self.display_list.max_y();
          self.document = Some(doc);
        }

        Task::none()
      }

      Message::WindowResized(width, height) => {
        self.width = width;
        self.height = height;
        self.relayout();
        Task::none()
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let browser_canvas = BrowserCanvas {
      display_list: &self.display_list,
      scroll_offset: self.scroll_offset,
      max_y: self.max_y,
      height: self.height,
    };

    let content = canvas(browser_canvas)
      .width(iced::Length::Fill)
      .height(iced::Length::Fill);

    container(content)
      .width(iced::Length::Fill)
      .height(iced::Length::Fill)
      .padding(10)
      .into()
  }

  pub fn theme(&self) -> iced::Theme {
    iced::Theme::Light
  }
}
