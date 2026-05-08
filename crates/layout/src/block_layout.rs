use crate::display_list::DisplayList;
use crate::layout::{HSTEP, decode_entities};
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
  is_superscript: bool,
  is_preformatted: bool,
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
      is_superscript: false,
      is_preformatted: false,
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
    let css_width = node
      .style()
      .get("width")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());
    let css_height = node
      .style()
      .get("height")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok());
    drop(node);

    if let Some(w) = css_width {
      self.width = w;
    }

    let mode = self.layout_mode();

    if mode == LayoutMode::Block {
      let children_nodes: Vec<Rc<RefCell<Node>>> = self
        .node
        .borrow()
        .children()
        .iter()
        .map(Rc::clone)
        .collect();

      let mut prev_bottom = None;
      for child_node in children_nodes {
        let mut child = BlockLayout::new(child_node);
        child.layout(self.x, self.y, self.width, prev_bottom, font_cache);
        prev_bottom = Some(child.y + child.height);
        self.children.push(LayoutChild::Block(Box::new(child)));
      }
    } else {
      self.cursor_x = 0.0;
      self.new_line();
      let node_rc = Rc::clone(&self.node);
      self.recurse(&node_rc, font_cache);

      let mut prev_bottom = None;
      for child in &mut self.children {
        if let LayoutChild::Line(line) = child {
          line.layout(self.x, self.y, self.width, prev_bottom);
          prev_bottom = Some(line.y + line.height);
        }
      }
    }

    if let Some(h) = css_height {
      self.height = h;
    } else {
      self.height = self
        .children
        .iter()
        .map(|c| match c {
          LayoutChild::Block(b) => b.height,
          LayoutChild::Line(l) => l.height,
        })
        .sum();
    }
  }

  fn new_line(&mut self) {
    self.cursor_x = 0.0;
    self
      .children
      .push(LayoutChild::Line(Box::new(LineLayout::new(Rc::clone(
        &self.node,
      )))));
  }

  fn word(
    &mut self,
    node_rc: &Rc<RefCell<Node>>,
    word: String,
    font_cache: &mut HashMap<FontKey, Font>,
  ) {
    let mut text_layout = TextLayout::new(Rc::clone(node_rc), word.clone(), self.is_superscript);

    let mut get_font_fn = |family_str: String, weight: Weight, style: Style, size: f32| -> Font {
      let key = FontKey {
        family: family_str.clone(),
        weight,
        style,
        size_pts: size as u32,
      };
      *font_cache.entry(key).or_insert_with(|| {
        let family = match family_str.to_lowercase().as_str() {
          "monospace" | "courier" | "consolas" => iced::font::Family::Monospace,
          "serif" | "times" | "georgia" => iced::font::Family::Serif,
          _ => iced::font::Family::SansSerif,
        };
        Font {
          family,
          weight,
          style,
          ..Font::DEFAULT
        }
      })
    };

    text_layout.measure(&mut get_font_fn);

    let space_w = {
      let space_para = GraphicsParagraph::with_text(AdvancedText {
        content: " ",
        bounds: Size::INFINITY,
        size: Pixels(text_layout.size),
        line_height: LineHeight::default(),
        font: text_layout.font,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
      });
      space_para.min_bounds().width
    };

    if !self.is_preformatted && self.cursor_x + text_layout.width > self.width - HSTEP {
      self.new_line();
    }

    if let Some(LayoutChild::Line(line)) = self.children.last_mut() {
      text_layout.x = self.x + self.cursor_x;
      line.children.push(text_layout.clone());
    }

    self.cursor_x += text_layout.width + space_w;
  }

  fn recurse(&mut self, node_rc: &Rc<RefCell<Node>>, font_cache: &mut HashMap<FontKey, Font>) {
    let node = node_rc.borrow();
    match &*node {
      Node::Text(text) => {
        let decoded = decode_entities(&text.text);
        if self.is_preformatted {
          let lines: Vec<&str> = decoded.split('\n').collect();
          for (i, line) in lines.iter().enumerate() {
            if i > 0 {
              self.new_line();
            }
            if !line.is_empty() || i < lines.len() - 1 {
              self.word(node_rc, line.to_string(), font_cache);
            }
          }
        } else {
          let words: Vec<&str> = decoded.split_whitespace().collect();
          for word in words {
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

  fn open_tag(&mut self, tag: &str) {
    match tag {
      "br" | "p" => self.new_line(),
      "sup" => self.is_superscript = true,
      "pre" => {
        self.new_line();
        self.is_preformatted = true;
      }
      _ => (),
    }
  }

  fn close_tag(&mut self, tag: &str) {
    match tag {
      "p" => self.new_line(),
      "sup" => self.is_superscript = false,
      "pre" => self.is_preformatted = false,
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

  None
}
