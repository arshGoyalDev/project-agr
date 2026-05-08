use crate::display_list::DisplayList;
use crate::text_layout::TextLayout;

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

  pub fn paint(&self, cmds: &mut DisplayList) {
    for word in &self.children {
      word.paint(cmds);
    }
  }
}
