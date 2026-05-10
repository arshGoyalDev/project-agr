use crate::display_list::DisplayList;
use crate::layout::{HSTEP, VSTEP, decode_entities};
use crate::line_layout::LineLayout;
use crate::text_layout::TextLayout;
use html_parser::Node;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::{Paragraph, Text as AdvancedText};
use iced::font::{Font, Style, Weight};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Color, Pixels, Size, alignment};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontKey {
  pub family: String,
  pub weight: Weight,
  pub style: Style,
  pub size_pts: u32,
}

pub enum LayoutChild {
  Block(Box<BlockLayout>),
  Line(Box<LineLayout>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
  Block,
  Inline,
}

pub struct BlockLayout {
  pub node: Rc<RefCell<Node>>,
  pub children: Vec<LayoutChild>,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,

  cursor_x: f32,
  cursor_y: f32,
  is_center: bool,
  is_superscript: bool,
  is_preformatted: bool,
  needs_space: bool,
  current_line: Vec<TextLayout>,
}

impl BlockLayout {
  pub fn new(node: Rc<RefCell<Node>>) -> Self {
    Self {
      node,
      children: vec![],
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      cursor_x: 0.0,
      cursor_y: 0.0,
      is_center: false,
      is_superscript: false,
      is_preformatted: false,
      needs_space: false,
      current_line: vec![],
    }
  }

  pub fn layout_mode(&self) -> LayoutMode {
    let node = self.node.borrow();
    match &*node {
      Node::Text(_) => LayoutMode::Inline,
      Node::Element(e) => {
        let has_block_child = e.children.iter().any(|c| {
          c.borrow()
            .style()
            .get("display")
            .map(|s| s.as_str())
            .unwrap_or("inline")
            == "block"
        });

        if has_block_child {
          LayoutMode::Block
        } else if !e.children.is_empty() {
          LayoutMode::Inline
        } else {
          LayoutMode::Block
        }
      }
    }
  }

  pub fn layout(
    &mut self,
    parent_x: f32,
    parent_y: f32,
    parent_width: f32,
    previous_bottom: Option<f32>,
    font_cache: &mut HashMap<FontKey, Font>,
  ) {
    self.x = parent_x;
    self.y = previous_bottom.unwrap_or(parent_y);
    self.width = parent_width;

    let node = self.node.borrow();
    let style_map = node.style();
    let css_width = style_map
      .get("width")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());
    let css_height = style_map
      .get("height")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());

    let children_nodes: Vec<Rc<RefCell<Node>>> = node.children().iter().map(Rc::clone).collect();
    drop(node);

    if let Some(w) = css_width {
      self.width = w;
    }

    let mode = self.layout_mode();

    if mode == LayoutMode::Block {
      let mut prev_bottom = None;
      let mut calc_height = 0.0;
      for child_node in children_nodes {
        let mut child = BlockLayout::new(child_node);
        child.layout(self.x, self.y, self.width, prev_bottom, font_cache);
        prev_bottom = Some(child.y + child.height);
        calc_height += child.height;
        self.children.push(LayoutChild::Block(Box::new(child)));
      }

      if let Some(h) = css_height {
        self.height = h;
      } else {
        self.height = calc_height;
      }
    } else {
      self.cursor_x = 0.0;
      self.cursor_y = 0.0;
      self.needs_space = false;
      self.current_line.clear();

      let node_rc = Rc::clone(&self.node);
      self.recurse(&node_rc, font_cache);
      self.flush();

      if let Some(h) = css_height {
        self.height = h;
      } else {
        self.height = self.cursor_y;
      }
    }
  }

  fn recurse(&mut self, node_rc: &Rc<RefCell<Node>>, font_cache: &mut HashMap<FontKey, Font>) {
    let node = node_rc.borrow();
    match &*node {
      Node::Text(text) => {
        let decoded = decode_entities(&text.text);
        if self.is_preformatted {
          for line in decoded.split('\n') {
            for word in line.split(' ') {
              self.word(node_rc, word.to_string(), font_cache);
            }
            self.flush();
          }
        } else {
          let words: Vec<&str> = decoded.split_whitespace().collect();
          for (i, word) in words.iter().enumerate() {
            if i == 0 && !decoded.starts_with(|c: char| c.is_whitespace()) {
              self.needs_space = false;
            }
            self.word(node_rc, word.to_string(), font_cache);
          }
        }
      }
      Node::Element(element) => {
        if element.tag == "script" {
          return;
        }
        let tag = element.tag.clone();
        let children: Vec<Rc<RefCell<Node>>> = element.children.iter().map(Rc::clone).collect();
        drop(node);

        self.open_tag(&tag);
        for child in &children {
          self.recurse(child, font_cache);
        }
        self.close_tag(&tag);
      }
    }
  }

  pub fn get_node(&self, x: f32, y: f32) -> Option<Rc<RefCell<Node>>> {
    if x < self.x || x > self.x + self.width || y < self.y || y > self.y + self.height {
      return None;
    }
    for child in &self.children {
      match child {
        LayoutChild::Block(b) => {
          if let Some(node) = b.get_node(x, y) {
            return Some(node);
          }
        }
        LayoutChild::Line(l) => {
          if let Some(node) = l.get_node(x, y) {
            return Some(node);
          }
        }
      }
    }
    Some(Rc::clone(&self.node)) 
  }

  fn word(
    &mut self,
    node_rc: &Rc<RefCell<Node>>,
    word: String,
    font_cache: &mut HashMap<FontKey, Font>,
  ) {
    let node = node_rc.borrow();
    let style_map = node.style();

    let weight = match style_map.get("font-weight").map(|s| s.as_str()) {
      Some("bold") => Weight::Bold,
      _ => Weight::Normal,
    };

    let style = match style_map.get("font-style").map(|s| s.as_str()) {
      Some("italic") | Some("oblique") => Style::Italic,
      _ => Style::Normal,
    };

    let mut size: f32 = style_map
      .get("font-size")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
      .map(|px| px * 0.75)
      .unwrap_or(12.0);

    size = size.max(1.0);

    let color = style_map
      .get("color")
      .and_then(|s| parse_css_color(s))
      .unwrap_or(Color::BLACK);

    let family_str = style_map
      .get("font-family")
      .cloned()
      .unwrap_or_else(|| "sans-serif".to_string());

    drop(node);

    let font = get_font(&family_str, weight, style, size, font_cache);

    let make_paragraph = |content: &str| {
      GraphicsParagraph::with_text(AdvancedText {
        content,
        bounds: Size::INFINITY,
        size: Pixels(size),
        line_height: LineHeight::default(),
        font,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
      })
    };

    let word_size = make_paragraph(&word).min_bounds();
    let space_size = make_paragraph(" ").min_bounds();

    if word.is_empty() {
      self.cursor_x += space_size.width;
      return;
    }

    let space_advance = if self.needs_space {
      space_size.width
    } else {
      0.0
    };

    if !self.is_preformatted && self.cursor_x + space_advance + word_size.width > self.width - HSTEP
    {
      self.flush();
      self.current_line.push(TextLayout {
        node: node_rc.clone(),
        width: word_size.width,
        x: self.cursor_x,
        y: 0.0,
        word,
        font,
        size,
        color,
        is_superscript: self.is_superscript,
      });
      self.cursor_x += word_size.width;
    } else {
      self.current_line.push(TextLayout {
        node: node_rc.clone(),
        width: word_size.width,
        x: if self.is_superscript {
          self.cursor_x + space_advance - space_size.width
        } else {
          self.cursor_x + space_advance
        },
        y: 0.0,
        word,
        font,
        size,
        color,
        is_superscript: self.is_superscript,
      });
      self.cursor_x += space_advance + word_size.width;
    }

    self.needs_space = true;
  }

  fn flush(&mut self) {
    if self.current_line.is_empty() {
      return;
    }

    let max_ascent = self
      .current_line
      .iter()
      .map(|i| i.size * 0.8)
      .fold(0.0_f32, f32::max);
    let baseline = self.cursor_y + 1.25 * max_ascent;

    let line_width = self.cursor_x - HSTEP;
    let offset = if self.is_center {
      (self.width - line_width) / 2.0 - HSTEP
    } else {
      0.0
    };

    for item in &mut self.current_line {
      item.x = self.x + item.x + offset;
      item.y = if item.is_superscript {
        self.y + baseline - item.size * 2.0
      } else {
        self.y + baseline - item.size
      };
    }

    let max_descent = self
      .current_line
      .iter()
      .map(|i| i.size * 0.2)
      .fold(0.0_f32, f32::max);
    let line_height = (baseline + 1.25 * max_descent) - self.cursor_y;

    self.cursor_y = baseline + 1.25 * max_descent;
    self.cursor_x = 0.0;
    self.needs_space = false;

    let finalized_line = std::mem::take(&mut self.current_line);
    self
      .children
      .push(LayoutChild::Line(Box::new(LineLayout::new(
        self.x,
        self.y + self.cursor_y - line_height,
        self.width,
        line_height,
        finalized_line,
      ))));
  }

  fn open_tag(&mut self, tag: &str) {
    match tag {
      "br" => self.flush(),
      "p" => {
        self.flush();
        self.cursor_y += VSTEP;
      }
      "center" => {
        self.flush();
        self.is_center = true;
      }
      "sup" => self.is_superscript = true,
      "pre" => {
        self.flush();
        self.cursor_y += VSTEP;
        self.is_preformatted = true;
      }
      _ => (),
    }
  }

  fn close_tag(&mut self, tag: &str) {
    match tag {
      "p" => {
        self.flush();
        self.cursor_y += VSTEP;
      }
      "center" => {
        self.flush();
        self.is_center = false;
      }
      "sup" => self.is_superscript = false,
      "pre" => {
        self.flush();
        self.is_preformatted = false;
      }
      _ => (),
    }
  }

  pub fn paint(&self, cmds: &mut DisplayList) {
    let node = self.node.borrow();
    let bgcolor = node
      .style()
      .get("background-color")
      .cloned()
      .unwrap_or_else(|| "transparent".to_string());

    if bgcolor != "transparent" {
      if let Some(color) = parse_css_color(&bgcolor) {
        cmds.add_rect(
          self.x,
          self.y,
          self.x + self.width,
          self.y + self.height,
          color,
        );
      }
    }

    for child in &self.children {
      match child {
        LayoutChild::Block(b) => b.paint(cmds),
        LayoutChild::Line(l) => l.paint(cmds),
      }
    }
  }
}

fn get_font(
  family_str: &str,
  weight: Weight,
  style: Style,
  size: f32,
  font_cache: &mut HashMap<FontKey, Font>,
) -> Font {
  let key = FontKey {
    family: family_str.to_string(),
    weight,
    style,
    size_pts: size as u32,
  };

  *font_cache.entry(key).or_insert_with(|| {
    let family = match family_str.to_lowercase().as_str() {
      "monospace" | "courier" | "consolas" => iced::font::Family::Monospace,
      "serif" | "times" | "times new roman" | "georgia" => iced::font::Family::Serif,
      "cursive" | "comic sans ms" => iced::font::Family::Cursive,
      "fantasy" | "impact" => iced::font::Family::Fantasy,
      _ => iced::font::Family::SansSerif,
    };
    Font {
      family,
      weight,
      style,
      ..Font::DEFAULT
    }
  })
}

pub fn parse_css_color(s: &str) -> Option<Color> {
  let s = s.trim();

  match s {
    "black" => return Some(Color::BLACK),
    "white" => return Some(Color::WHITE),
    "red" => return Some(Color::from_rgb(1.0, 0.0, 0.0)),
    "green" => return Some(Color::from_rgb(0.0, 0.502, 0.0)),
    "blue" => return Some(Color::from_rgb(0.0, 0.0, 1.0)),
    "lightblue" => return Some(Color::from_rgb(0.678, 0.847, 0.902)),
    "gray" | "grey" => return Some(Color::from_rgb(0.502, 0.502, 0.502)),
    "yellow" => return Some(Color::from_rgb(1.0, 1.0, 0.0)),
    "orange" => return Some(Color::from_rgb(1.0, 0.647, 0.0)),
    "purple" => return Some(Color::from_rgb(0.502, 0.0, 0.502)),
    "transparent" => return None,
    _ => {}
  }

  if s.starts_with('#') && s.len() == 7 {
    let r = u8::from_str_radix(&s[1..3], 16).ok()?;
    let g = u8::from_str_radix(&s[3..5], 16).ok()?;
    let b = u8::from_str_radix(&s[5..7], 16).ok()?;
    return Some(Color::from_rgb(
      r as f32 / 255.0,
      g as f32 / 255.0,
      b as f32 / 255.0,
    ));
  }

  if s.starts_with('#') && s.len() == 4 {
    let r = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
    let g = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
    let b = u8::from_str_radix(&s[3..4].repeat(2), 16).ok()?;
    return Some(Color::from_rgb(
      r as f32 / 255.0,
      g as f32 / 255.0,
      b as f32 / 255.0,
    ));
  }

  None
}
