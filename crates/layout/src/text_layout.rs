use crate::block_layout::parse_css_color;
use crate::display_list::DisplayList;
use html_parser::Node;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::{Paragraph, Text as AdvancedText};
use iced::font::{Font, Style, Weight};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Color, Pixels, Size, alignment};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextLayout {
  pub node: Rc<RefCell<Node>>,
  pub word: String,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub font: Font,
  pub size: f32,
  pub color: Color,
  pub ascent: f32,
  pub descent: f32,
  pub is_superscript: bool,
}

impl TextLayout {
  pub fn new(node: Rc<RefCell<Node>>, word: String, is_superscript: bool) -> Self {
    Self {
      node,
      word,
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      font: Font::DEFAULT,
      size: 16.0,
      color: Color::BLACK,
      ascent: 0.0,
      descent: 0.0,
      is_superscript,
    }
  }

  pub fn measure(&mut self, get_font: &mut impl FnMut(String, Weight, Style, f32) -> Font) {
    let node = self.node.borrow();
    let style_map = node.style();

    let weight = match style_map.get("font-weight").map(|s| s.as_str()) {
      Some("bold") => Weight::Bold,
      _ => Weight::Normal,
    };

    let style = match style_map.get("font-style").map(|s| s.as_str()) {
      Some("italic") | Some("oblique") => Style::Italic,
      _ => Style::Normal,
    };

    let family = style_map
      .get("font-family")
      .cloned()
      .unwrap_or_else(|| "sans-serif".to_string());

    let mut size: f32 = style_map
      .get("font-size")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
      .map(|px| px * 0.75)
      .unwrap_or(12.0);

    size = size.max(1.0);
    if self.is_superscript {
      size /= 2.0;
    }

    self.color = style_map
      .get("color")
      .and_then(|s| parse_css_color(s))
      .unwrap_or(Color::BLACK);

    self.size = size;
    self.font = get_font(family, weight, style, size);

    let paragraph = GraphicsParagraph::with_text(AdvancedText {
      content: &self.word,
      bounds: Size::INFINITY,
      size: Pixels(size),
      line_height: LineHeight::default(),
      font: self.font,
      horizontal_alignment: alignment::Horizontal::Left,
      vertical_alignment: alignment::Vertical::Top,
      shaping: Shaping::Basic,
      wrapping: Wrapping::None,
    });

    let bounds = paragraph.min_bounds();
    self.width = bounds.width;
    self.height = bounds.height;

    // Approximate standard metrics since iced abstracts this away
    self.ascent = size * 0.8;
    self.descent = size * 0.2;
  }

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
