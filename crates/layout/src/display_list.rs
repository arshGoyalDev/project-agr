use iced::Color;
use iced::font::Font;

#[derive(Debug, Clone)]
pub struct DrawText {
  pub x: f32,
  pub y: f32,
  pub word: String,
  pub font: Font,
  pub size: f32,
  pub color: Color,
  pub bottom: f32,
}

#[derive(Debug, Clone)]
pub struct DrawRect {
  pub x1: f32,
  pub y1: f32,
  pub x2: f32,
  pub y2: f32,
  pub color: Color,
  pub bottom: f32,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
  Text(DrawText),
  Rect(DrawRect),
}

impl DrawCommand {
  pub fn top(&self) -> f32 {
    match self {
      DrawCommand::Text(t) => t.y,
      DrawCommand::Rect(r) => r.y1,
    }
  }
  pub fn bottom(&self) -> f32 {
    match self {
      DrawCommand::Text(t) => t.bottom,
      DrawCommand::Rect(r) => r.bottom,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DisplayList {
  items: Vec<DrawCommand>,
}

impl DisplayList {
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub fn add_text(&mut self, x: f32, y: f32, word: String, font: Font, size: f32, color: Color) {
    let bottom = y + size * 1.4;
    self.items.push(DrawCommand::Text(DrawText {
      x,
      y,
      word,
      font,
      size,
      color,
      bottom,
    }));
  }

  pub fn add_rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
    self.items.push(DrawCommand::Rect(DrawRect {
      x1,
      y1,
      x2,
      y2,
      color,
      bottom: y2,
    }));
  }

  pub fn extend(&mut self, other: &DisplayList) {
    self.items.extend(other.items.iter().cloned());
  }

  pub fn items(&self) -> &[DrawCommand] {
    &self.items
  }

  pub fn max_y(&self) -> f32 {
    self
      .items
      .iter()
      .map(|c| c.bottom())
      .fold(0.0_f32, f32::max)
  }
}
