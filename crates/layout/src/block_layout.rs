use crate::display_list::DisplayList;
use crate::layout::{HSTEP, VSTEP, BLOCK_ELEMENTS, PRE_BG, decode_entities};
use html_parser::Node;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::Paragraph;
use iced::advanced::text::Text as AdvancedText;
use iced::alignment;
use iced::font::{Font, Style, Weight};
use iced::widget::text::Wrapping;
use iced::widget::text::{LineHeight, Shaping};
use iced::{Pixels, Size};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontKey {
  pub weight: Weight,
  pub style: Style,
}

struct LineItem {
  x: f32,
  word: String,
  font: Font,
  size: f32,
  is_superscript: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LayoutMode {
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

  // Inline layout state (only used when mode == Inline)
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

  fn layout_mode(&self) -> LayoutMode {
    let node = self.node.borrow();
    match &*node {
      Node::Text(_) => LayoutMode::Inline,
      Node::Element(e) => {
        // If any child Element has a block tag → block mode
        let has_block_child = e.children.iter().any(|c| {
          let c = c.borrow();
          match &*c {
            Node::Element(ce) => BLOCK_ELEMENTS.contains(&ce.tag.as_str()),
            _ => false,
          }
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
      // Build child BlockLayouts first, then lay each out
      let node = self.node.borrow();
      let children_nodes: Vec<Rc<RefCell<Node>>> =
        node.children().iter().map(|c| Rc::clone(c)).collect();
      drop(node);

      let mut previous_bottom: Option<f32> = None;
      for child_node in children_nodes {
        let mut child = BlockLayout::new(child_node, self.x, self.y, self.width, previous_bottom);
        child.layout();
        previous_bottom = Some(child.y + child.height);
        self.display_list.extend(&child.display_list);
        self.children.push(child);
      }

      self.height = self.children.iter().map(|c| c.height).sum();
    } else {
      // Inline mode
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
    if let Node::Element(e) = &*node {
      if e.tag == "pre" {
        cmds.add_rect(
          self.x,
          self.y,
          self.x + self.width,
          self.y + self.height,
          PRE_BG,
        );
      }
    }

    // Inline text commands are appended by paint_tree after the rect
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
              self.word(word.to_string());
            }
            self.flush();
          }
        } else {
          let words: Vec<&str> = decoded.split_whitespace().collect();
          for (i, word) in words.iter().enumerate() {
            if i == 0 && !decoded.starts_with(|c: char| c.is_whitespace()) {
              self.needs_space = false;
            }
            self.word(word.to_string());
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
        return; // already dropped node above
      }
    }
  }

  fn word(&mut self, word: String) {
    let font = self.get_font(self.weight, self.style);

    let make_paragraph = |content: &str, size: f32| {
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

    let word_size = make_paragraph(&word, self.size).min_bounds();
    let space_size = make_paragraph(" ", self.size).min_bounds();

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
        size: self.size,
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
        size: self.size,
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
    let baseline = self.cursor_y + 1.2 * max_ascent;

    let line_width = self.cursor_x - HSTEP;
    let offset = if self.is_center {
      (self.width - line_width) / 2.0 - HSTEP
    } else {
      0.0
    };

    for item in &self.line {
      let rel_x = item.x + offset;
      // absolute x = self.x (block's left edge) + rel_x
      let abs_x = self.x + rel_x;
      let y = if item.is_superscript {
        self.y + baseline - item.size * 2.0
      } else {
        self.y + baseline - item.size
      };
      self
        .display_list
        .add_text(abs_x, y, item.word.clone(), item.font, item.size);
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

  fn get_font(&mut self, weight: Weight, style: Style) -> Font {
    let key = FontKey { weight, style };
    let font_ref = self.font_cache.entry(key).or_insert(Font {
      weight,
      style,
      ..Font::DEFAULT
    });
    *font_ref
  }

  fn open_tag(&mut self, tag: &str) {
    match tag {
      "i" => self.style = Style::Italic,
      "b" => self.weight = Weight::Bold,
      "small" => self.size -= 2.0,
      "big" => self.size += 4.0,
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
      "i" => self.style = Style::Normal,
      "b" => self.weight = Weight::Normal,
      "small" => self.size += 2.0,
      "big" => self.size -= 4.0,
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