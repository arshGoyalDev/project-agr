use crate::display_list::DisplayList;
use iced::Color;
use iced::font::Font;

#[derive(Clone, Debug)]
pub struct TextLayout {
  pub word: String,
  pub font: Font,
  pub size: f32,
  pub color: Color,
  pub x: f32,
  pub y: f32,
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
}
