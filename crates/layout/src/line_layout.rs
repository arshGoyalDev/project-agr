use crate::display_list::DisplayList;
use crate::text_layout::TextLayout;

use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

pub struct LineLayout {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub children: Vec<TextLayout>,
}

impl LineLayout {
  pub fn new(x: f32, y: f32, width: f32, height: f32, children: Vec<TextLayout>) -> Self {
    Self {
      x,
      y,
      width,
      height,
      children,
    }
  }

  pub fn get_node(&self, x: f32, y: f32) -> Option<Rc<RefCell<Node>>> {
    if y < self.y || y > self.y + self.height {
      return None;
    }
    for word in &self.children {
      if let Some(node) = word.get_node(x, y) {
        return Some(node);
      }
    }
    None
  }

  pub fn paint(&self, cmds: &mut DisplayList) {
    for word in &self.children {
      word.paint(cmds);
    }
  }
}
