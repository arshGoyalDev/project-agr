use crate::display_list::DisplayList;
use crate::layout::{HSTEP, VSTEP, decode_entities};
use html_parser::Node;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::Paragraph;
use iced::advanced::text::Text as AdvancedText;
use iced::alignment;
use iced::font::{Font, Style, Weight};
use iced::widget::text::Wrapping;
use iced::widget::text::{LineHeight, Shaping};
use iced::{Color, Pixels, Size};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey {
  family: String,
  weight: Weight,
  style: Style,
  size_pts: u32,
}

struct LineItem {
  x: f32,
  word: String,
  font: Font,
  size: f32,
  color: Color,
  is_superscript: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
  Block,
  Inline,
}

pub struct BlockLayout {
  pub node: Rc<RefCell<Node>>,
  pub children: Vec<BlockLayout>,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub display_list: DisplayList,

  cursor_x: f32,
  cursor_y: f32,
  weight: Weight,
  style: Style,
  size: f32,
  is_center: bool,
  is_superscript: bool,
  is_preformatted: bool,
  needs_space: bool,
  line: Vec<LineItem>,
  font_cache: HashMap<FontKey, Font>,
}

impl BlockLayout {
  pub fn new(
    node: Rc<RefCell<Node>>,
    parent_x: f32,
    parent_y: f32,
    parent_width: f32,
    previous_bottom: Option<f32>,
  ) -> Self {
    let y = previous_bottom.unwrap_or(parent_y);
    Self {
      node,
      children: vec![],
      x: parent_x,
      y,
      width: parent_width,
      height: 0.0,
      display_list: DisplayList::new(),
      cursor_x: 0.0,
      cursor_y: 0.0,
      weight: Weight::Normal,
      style: Style::Normal,
      size: 16.0,
      is_center: false,
      is_superscript: false,
      is_preformatted: false,
      needs_space: false,
      line: vec![],
      font_cache: HashMap::new(),
    }
  }

  pub fn layout_mode(&self) -> LayoutMode {
    let node = self.node.borrow();
    match &*node {
      Node::Text(_) => LayoutMode::Inline,
      Node::Element(e) => {
        let has_block_child = e.children.iter().any(|c| {
          let child_node = c.borrow();

          let display = child_node
            .style()
            .get("display")
            .map(|s| s.as_str())
            .unwrap_or("inline");

          display == "block"
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

  pub fn layout(&mut self) {
    let mode = self.layout_mode();

    if mode == LayoutMode::Block {
      let node = self.node.borrow();
      let style_map = node.style();

      let css_width = style_map
        .get("width")
        .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());

      let css_height = style_map
        .get("height")
        .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());

      let children_nodes: Vec<Rc<RefCell<Node>>> =
        node.children().iter().map(|c| Rc::clone(c)).collect();
      drop(node);

      if let Some(w) = css_width {
        self.width = w;
      }

      let mut previous_bottom: Option<f32> = None;
      for child_node in children_nodes {
        let mut child = BlockLayout::new(child_node, self.x, self.y, self.width, previous_bottom);
        child.layout();
        previous_bottom = Some(child.y + child.height);
        self.display_list.extend(&child.display_list);
        self.children.push(child);
      }

      if let Some(h) = css_height {
        self.height = h;
      } else {
        self.height = self.children.iter().map(|c| c.height).sum();
      }
    } else {
      self.cursor_x = 0.0;
      self.cursor_y = 0.0;
      self.weight = Weight::Normal;
      self.style = Style::Normal;
      self.size = 16.0;
      self.line = vec![];

      let node_rc = Rc::clone(&self.node);
      self.recurse(&node_rc);
      self.flush();

      self.height = self.cursor_y;
    }
  }

  pub fn paint(&self) -> DisplayList {
    let mut cmds = DisplayList::new();

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

    if self.layout_mode() == LayoutMode::Inline {
      cmds.extend(&self.display_list);
    }

    cmds
  }

  fn recurse(&mut self, node_rc: &Rc<RefCell<Node>>) {
    let node = node_rc.borrow();
    match &*node {
      Node::Text(text) => {
        let decoded = decode_entities(&text.text);
        if self.is_preformatted {
          for line in decoded.split('\n') {
            for word in line.split(' ') {
              self.word(node_rc, word.to_string());
            }
            self.flush();
          }
        } else {
          let words: Vec<&str> = decoded.split_whitespace().collect();
          for (i, word) in words.iter().enumerate() {
            if i == 0 && !decoded.starts_with(|c: char| c.is_whitespace()) {
              self.needs_space = false;
            }
            self.word(node_rc, word.to_string());
          }
        }
      }
      Node::Element(element) => {
        if element.tag == "script" {
          return;
        }
        let tag = element.tag.clone();
        let children: Vec<Rc<RefCell<Node>>> =
          element.children.iter().map(|c| Rc::clone(c)).collect();
        drop(node);

        self.open_tag(&tag);
        for child in &children {
          self.recurse(child);
        }
        self.close_tag(&tag);
        return;
      }
    }
  }

  fn word(&mut self, node_rc: &Rc<RefCell<Node>>, word: String) {
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

    let size: f32 = style_map
      .get("font-size")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
      .map(|px| px * 0.75)
      .unwrap_or(12.0);

    let color = style_map
      .get("color")
      .and_then(|s| parse_css_color(s))
      .unwrap_or(Color::BLACK);

    let family = style_map
      .get("font-family")
      .cloned()
      .unwrap_or_else(|| "sans-serif".to_string());

    drop(node);

    let font = self.get_font(family, weight, style, size);

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
      self.line.push(LineItem {
        x: self.cursor_x,
        word,
        font,
        size,
        color,
        is_superscript: self.is_superscript,
      });
      self.cursor_x += word_size.width;
    } else {
      self.line.push(LineItem {
        x: if self.is_superscript {
          self.cursor_x + space_advance - space_size.width
        } else {
          self.cursor_x + space_advance
        },
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
    if self.line.is_empty() {
      return;
    }

    let max_ascent = self
      .line
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

    for item in &self.line {
      let abs_x = self.x + item.x + offset;
      let y = if item.is_superscript {
        self.y + baseline - item.size * 2.0
      } else {
        self.y + baseline - item.size
      };
      self.display_list.add_text(
        abs_x,
        y,
        item.word.clone(),
        item.font,
        item.size,
        item.color,
      );
    }

    let max_descent = self
      .line
      .iter()
      .map(|i| i.size * 0.2)
      .fold(0.0_f32, f32::max);
    self.cursor_y = baseline + 1.25 * max_descent;
    self.cursor_x = 0.0;
    self.needs_space = false;
    self.line.clear();
  }

  fn get_font(&mut self, family_str: String, weight: Weight, style: Style, size: f32) -> Font {
    let key = FontKey {
      family: family_str.to_string(),
      weight,
      style,
      size_pts: size as u32,
    };

    *self.font_cache.entry(key).or_insert_with(|| {
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
      "sup" => {
        self.is_superscript = true;
        self.size /= 2.0;
      }
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
      "sup" => {
        self.is_superscript = false;
        self.size *= 2.0;
      }
      "pre" => {
        self.flush();
        self.is_preformatted = false;
      }
      _ => (),
    }
  }
}

pub fn parse_css_color(s: &str) -> Option<Color> {
  let s = s.trim();

  match s {
    "black" => return Some(Color::BLACK),
    "white" => return Some(Color::WHITE),
    "red" => return Some(Color::from_rgb(1.0, 0.0, 0.0)),
    "green" => return Some(Color::from_rgb(0.0, 0.502, 0.0)),
    "blue" => return Some(Color::from_rgb(0.0, 0.0, 1.0)),
    "gray" | "grey" => return Some(Color::from_rgb(0.502, 0.502, 0.502)),
    "yellow" => return Some(Color::from_rgb(1.0, 1.0, 0.0)),
    "orange" => return Some(Color::from_rgb(1.0, 0.647, 0.0)),
    "purple" => return Some(Color::from_rgb(0.502, 0.0, 0.502)),
    "transparent" => return None,
    _ => {}
  }
  // #rrggbb
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
  // #rgb shorthand
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
