use iced::font::Font;

#[derive(Debug, Clone)]
pub struct DisplayList {
  items: Vec<DisplayItem>,
}

#[derive(Debug, Clone)]
pub struct DisplayItem {
  pub x: f32,
  pub y: f32,
  pub word: String,
  pub font: Font,
  pub size: f32,
}

impl DisplayList {
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub fn add_item(&mut self, x: f32, y: f32, word: String, font: Font, size: f32) {
    self.items.push(DisplayItem {
      x,
      y,
      word,
      font,
      size,
    });
  }

  pub fn items(&self) -> &[DisplayItem] {
    &self.items
  }
}
