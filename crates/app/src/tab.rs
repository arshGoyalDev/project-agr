use html_parser::Node;
use layout::{DisplayList, DocumentLayout};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Tab {
  pub url: String,
  pub history: Vec<String>,
  pub history_index: usize,
  pub tree: Option<Rc<RefCell<Node>>>,
  pub document: Option<DocumentLayout>,
  pub display_list: DisplayList,
  pub scroll_offset: f32,
  pub max_y: f32,
  pub title: String,
}

impl Tab {
  pub fn new(url: String) -> Self {
    Self {
      url: url.clone(),
      history: vec![url],
      history_index: 0,
      tree: None,
      document: None,
      display_list: DisplayList::new(),
      scroll_offset: 0.0,
      max_y: 0.0,
      title: String::new(),
    }
  }
}
