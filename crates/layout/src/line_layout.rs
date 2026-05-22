use crate::display_list::DisplayList;
use crate::input_layout::InputLayout;
use crate::text_layout::TextLayout;

use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub enum InlineLayout {
  Text(TextLayout),
  Input(InputLayout),
}

impl InlineLayout {
  pub fn paint(&self, cmds: &mut DisplayList) {
    match self {
      InlineLayout::Text(t) => t.paint(cmds),
      InlineLayout::Input(i) => i.paint(cmds),
    }
  }

  pub fn get_node(&self, x: f32, y: f32) -> Option<Rc<RefCell<Node>>> {
    match self {
      InlineLayout::Text(t) => t.get_node(x, y),
      InlineLayout::Input(i) => i.get_node(x, y),
    }
  }

  // Helper methods to abstract away the underlying layout type
  pub fn x(&self) -> f32 {
    match self {
      InlineLayout::Text(t) => t.x,
      InlineLayout::Input(i) => i.x,
    }
  }

  pub fn set_x(&mut self, new_x: f32) {
    match self {
      InlineLayout::Text(t) => t.x = new_x,
      InlineLayout::Input(i) => i.x = new_x,
    }
  }

  pub fn set_y(&mut self, new_y: f32) {
    match self {
      InlineLayout::Text(t) => t.y = new_y,
      InlineLayout::Input(i) => i.y = new_y,
    }
  }

  pub fn size(&self) -> f32 {
    match self {
      InlineLayout::Text(t) => t.size,
      InlineLayout::Input(i) => i.size,
    }
  }

  pub fn is_superscript(&self) -> bool {
    match self {
      InlineLayout::Text(t) => t.is_superscript,
      InlineLayout::Input(_) => false,
    }
  }
}

pub struct LineLayout {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub children: Vec<InlineLayout>,
}

impl LineLayout {
  pub fn new(x: f32, y: f32, width: f32, height: f32, children: Vec<InlineLayout>) -> Self {
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
