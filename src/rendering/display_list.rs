#[derive(Debug, Clone)]
pub struct DisplayList {
  items: Vec<DisplayItem>,
}

#[derive(Debug, Clone)]
pub struct DisplayItem {
  pub x: f32,
  pub y: f32,
  pub character: char,
}

impl DisplayList {
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub fn add_item(&mut self, x: f32, y: f32, character: char) {
    self.items.push(DisplayItem { x, y, character });
  }

  pub fn items(&self) -> &[DisplayItem] {
    &self.items
  }
}