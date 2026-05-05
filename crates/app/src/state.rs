use iced::widget::{canvas, container};
use iced::{Element, Subscription, Task, window};

use css_parser::{CSSParser, style};
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

        if url_handler.view_source {
          if let Some(node) = &self.tree {
            let highlighted = syntax_highlight(node);
            let mut html_parser = HTMLParser::new(highlighted);
            self.tree = Some(html_parser.parse());
          }
        }

        if let Some(node) = &self.tree {
          let default_css = include_str!("../../../browser.css").to_string();
          let mut css_parser = CSSParser::new(&default_css);
          let mut rules = css_parser.parse();

          let mut links = Vec::new();
          find_stylesheet_links(node, &mut links);

          for link in links {
            let resolved_url = url_handler.resolve(&link);
            let mut style_handler = URLHandler::default();
            style_handler.init(resolved_url, false);

            if let Ok(css_body) = style_handler.request() {
              let mut linked_parser = CSSParser::new(&css_body);
              rules.extend(linked_parser.parse());
            }
          }

          rules.sort_by(|a, b| a.selector.priority().cmp(&b.selector.priority()));

          style(node, &rules);
        }

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

fn find_stylesheet_links(node_rc: &Rc<RefCell<Node>>, links: &mut Vec<String>) {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "link" {
      if let Some(rel) = e.attributes.get("rel") {
        if rel == "stylesheet" {
          if let Some(href) = e.attributes.get("href") {
            links.push(href.clone());
          }
        }
      }
    }
  }

  for child in node.children() {
    find_stylesheet_links(child, links);
  }
}
