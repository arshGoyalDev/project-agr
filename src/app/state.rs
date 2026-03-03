use iced::widget::{canvas, container};
use iced::{Element, Subscription, Task, window};

use crate::app::Message;
use crate::net::URLHandler;
use crate::rendering::{DisplayList, HTMLParser, Layout, print_tree, syntax_highlight};
use crate::ui::BrowserCanvas;
use crate::utils::Node;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

pub struct Browser {
  pub display_list: DisplayList,
  pub scroll_offset: f32,
  pub current_url: String,
  pub max_y: f32,
  pub width: f32,
  pub tree: Option<Rc<RefCell<Node>>>,
  pub height: f32,
}

impl Browser {
  pub fn new() -> (Self, Task<Message>) {
    let mut url = String::from("about:blank");
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
      Some(value) => {
        url = value.to_string();
      }
      _ => (),
    }

    (
      Self {
        display_list: DisplayList::new(),
        scroll_offset: 0.0,
        max_y: 0.0,
        current_url: String::from(url),
        tree: None,
        width: 0.0,
        height: 0.0,
      },
      Task::done(Message::LoadUrl()),
    )
  }

  pub fn subscription(&self) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size.width, size.height))
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

        // self.view_source = url_handler.view_source;

        match body_result {
          Ok(value) => {
            let mut html_parser = HTMLParser::new(value);
            self.tree = Some(html_parser.parse());
          }
          _ => (),
        }

        match &self.tree {
          Some(node) => {
            if url_handler.view_source {
              let highlighted = syntax_highlight(node);

              let mut html_parser = HTMLParser::new(highlighted);
              self.tree = Some(html_parser.parse());
            }
          }
          _ => (),
        }

        match &self.tree {
          Some(node) => {
            // print_tree(node, 0);
            let layout = Layout::new(node, self.width);
            self.display_list = layout.display_list;
          }
          _ => (),
        }

        self.max_y = self
          .display_list
          .items()
          .iter()
          .map(|item| item.y)
          .fold(0.0, f32::max);

        Task::none()
      }
      Message::WindowResized(width, height) => {
        self.width = width;
        self.height = height;

        match &self.tree {
          Some(node) => {
            let layout = Layout::new(node, self.width);
            self.display_list = layout.display_list;
          }
          _ => (),
        }

        self.max_y = self
          .display_list
          .items()
          .iter()
          .map(|item| item.y)
          .fold(0.0, f32::max);

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
