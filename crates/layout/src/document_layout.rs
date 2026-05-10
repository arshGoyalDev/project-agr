use crate::block_layout::{BlockLayout, FontKey};
use crate::display_list::DisplayList;
use crate::layout::{HSTEP, VSTEP};

use html_parser::Node;
use iced::font::Font;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct DocumentLayout {
  node: Rc<RefCell<Node>>,
  pub children: Vec<BlockLayout>,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  font_cache: HashMap<FontKey, Font>,
}

impl DocumentLayout {
  pub fn new(node: &Rc<RefCell<Node>>) -> Self {
    Self {
      node: Rc::clone(node),
      children: vec![],
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      font_cache: HashMap::new(),
    }
  }

  pub fn layout(&mut self, browser_width: f32) {
    self.children.clear();
    self.width = browser_width - 2.0 * HSTEP;
    self.x = HSTEP;
    self.y = VSTEP;

    let mut child = BlockLayout::new(Rc::clone(&self.node));

    child.layout(self.x, self.y, self.width, None, &mut self.font_cache);

    self.height = child.height;
    self.children.push(child);
  }

  pub fn get_node(&self, x: f32, y: f32) -> Option<Rc<RefCell<Node>>> {
    for child in &self.children {
      if let Some(node) = child.get_node(x, y) {
        return Some(node);
      }
    }
    None
  }

  pub fn paint(&self) -> DisplayList {
    let mut cmds = DisplayList::new();

    for child in &self.children {
      child.paint(&mut cmds);
    }
    cmds
  }
}
