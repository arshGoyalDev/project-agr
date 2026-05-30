use crate::display_list::DisplayList;

use html_parser::Node;
use iced::Color;
use iced::font::Font;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextLayout {
  pub node: Rc<RefCell<Node>>,
  pub word: String,
  pub font: Font,
  pub size: f32,
  pub color: Color,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub is_superscript: bool,
}

impl TextLayout {
  pub fn paint(&self, cmds: &mut DisplayList) {
    cmds.add_text(
      self.x,
      self.y,
      self.word.clone(),
      self.font,
      self.size,
      self.color,
    );
  }

  pub fn get_node(&self, px: f32, py: f32) -> Option<Rc<RefCell<Node>>> {
    if px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.size {
      Some(Rc::clone(&self.node))
    } else {
      None
    }
  }
}
