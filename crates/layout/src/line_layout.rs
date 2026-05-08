use crate::display_list::DisplayList;
use crate::text_layout::TextLayout;
use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

pub struct LineLayout {
  pub node: Rc<RefCell<Node>>,
  pub children: Vec<TextLayout>,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
}

impl LineLayout {
  pub fn new(node: Rc<RefCell<Node>>) -> Self {
    Self {
      node,
      children: vec![],
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
    }
  }

  pub fn layout(
    &mut self,
    parent_x: f32,
    parent_y: f32,
    parent_width: f32,
    previous_bottom: Option<f32>,
  ) {
    self.width = parent_width;
    self.x = parent_x;
    self.y = previous_bottom.unwrap_or(parent_y);

    if self.children.is_empty() {
      self.height = 0.0;
      return;
    }

    let max_ascent = self
      .children
      .iter()
      .map(|c| c.ascent)
      .fold(0.0_f32, f32::max);
    let max_descent = self
      .children
      .iter()
      .map(|c| c.descent)
      .fold(0.0_f32, f32::max);
    let baseline = self.y + 1.25 * max_ascent;

    for word in &mut self.children {
      word.y = if word.is_superscript {
        baseline - word.ascent * 2.0
      } else {
        baseline - word.ascent
      };
    }

    self.height = 1.25 * (max_ascent + max_descent);
  }

  pub fn paint(&self, cmds: &mut DisplayList) {
    for word in &self.children {
      word.paint(cmds);
    }
  }
}
