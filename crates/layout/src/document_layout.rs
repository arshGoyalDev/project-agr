use crate::block_layout::BlockLayout;
use crate::display_list::DisplayList;
use crate::layout::{HSTEP, VSTEP};

use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

pub struct DocumentLayout {
  node: Rc<RefCell<Node>>,
  pub children: Vec<BlockLayout>,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub display_list: DisplayList,
}

impl DocumentLayout {
  pub fn new(node: &Rc<RefCell<Node>>) -> Self {
    Self {
      node: Rc::clone(node),
      children: vec![],
      x: HSTEP,
      y: VSTEP,
      width: 0.0,
      height: 0.0,
      display_list: DisplayList::new(),
    }
  }

  pub fn layout(&mut self, browser_width: f32) {
    self.children.clear();
    self.display_list = DisplayList::new();

    self.x = HSTEP;
    self.y = VSTEP;
    self.width = browser_width - 2.0 * HSTEP;

    let mut child = BlockLayout::new(Rc::clone(&self.node), self.x, self.y, self.width, None);
    child.layout();
    self.height = child.height;
    self.children.push(child);
  }

  pub fn paint(&self) -> DisplayList {
    DisplayList::new()
  }
}
