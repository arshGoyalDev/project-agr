use crate::rendering::DisplayList;
use crate::utils::Node;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::Paragraph;
use iced::advanced::text::Text as AdvancedText;
use iced::alignment;
use iced::font::{Font, Style, Weight};
use iced::widget::text::Wrapping;
use iced::widget::text::{LineHeight, Shaping};
use iced::{Pixels, Size};

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

const HSTEP: f32 = 9.0;
const VSTEP: f32 = 15.0;

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

pub struct Layout {
  cursor_x: f32,
  cursor_y: f32,
  pub display_list: DisplayList,
  width: f32,
  line: Vec<LineItem>,
  weight: Weight,
  style: Style,
  size: f32,
  font_cache: HashMap<FontKey, Font>,
  is_center: bool,
  is_superscript: bool,
  is_preformatted: bool,
  needs_space: bool,
}

impl Layout {
  pub fn new(tree: &Rc<RefCell<Node>>, width: f32) -> Self {
    let mut layout = Self {
      display_list: DisplayList::new(),
      width,
      cursor_x: HSTEP,
      cursor_y: VSTEP,
      line: vec![],
      weight: Weight::Normal,
      style: Style::Normal,
      size: 16.0,
      font_cache: HashMap::new(),
      is_center: false,
      is_superscript: false,
      is_preformatted: false,
      needs_space: false,
    };

    layout.recurse(tree);
    layout.flush();
    layout
  }

  pub fn recurse(&mut self, node_rc: &Rc<RefCell<Node>>) {
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

        self.open_tag(&element.tag);

        for child in &element.children {
          self.recurse(&Rc::clone(child));
        }

        self.close_tag(&element.tag);
      }
    }
  }

  pub fn flush(&mut self) {
    if self.line.is_empty() {
      return;
    }

    let max_ascent = self.line.iter().map(|i| i.size * 0.8).fold(0.0, f32::max);
    let baseline = self.cursor_y + 1.2 * max_ascent;

    let line_width = self.cursor_x - HSTEP;
    let offset = if self.is_center {
      (self.width - line_width) / 2.0 - HSTEP
    } else {
      0.0
    };

    for item in &self.line {
      let y = if item.is_superscript {
        baseline - item.size * 2.0
      } else {
        baseline - item.size
      };

      self
        .display_list
        .add_item(item.x + offset, y, item.word.clone(), item.font, item.size)
    }

    self.cursor_y = baseline + 1.25 * max_ascent;
    self.cursor_x = HSTEP;
    self.needs_space = false;
    self.line.clear();
  }

  pub fn word(&mut self, word: String) {
    let font = self.get_font(self.weight, self.style);

    let make_paragraph = |content: &str| {
      GraphicsParagraph::with_text(AdvancedText {
        content,
        bounds: Size::INFINITY,
        size: Pixels(self.size),
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
        x: if self.is_superscript {
          self.cursor_x - space_size.width
        } else {
          self.cursor_x
        },
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

  pub fn get_font(&mut self, weight: Weight, style: Style) -> Font {
    let font_key = FontKey { weight, style };

    let font_ref = self.font_cache.entry(font_key).or_insert(Font {
      weight,
      style,
      ..Font::DEFAULT
    });

    *font_ref
  }

  pub fn open_tag(&mut self, tag: &str) {
    match tag {
      "i" => self.style = Style::Italic,
      "b" => self.weight = Weight::Bold,
      "small" => self.size -= 4.0,
      "big" => self.size += 4.0,
      "br" => self.flush(),
      "center" => {
        self.flush();
        self.is_center = true;
      }
      "sup" => {
        self.is_superscript = true;
        self.size /= 2.0;
      }
      "p" => {
        self.flush();
      }
      "pre" => {
        self.flush();
        self.cursor_y += VSTEP;
        self.is_preformatted = true;
      }
      _ => (),
    }
  }

  pub fn close_tag(&mut self, tag: &str) {
    match tag {
      "i" => self.style = Style::Normal,
      "b" => self.weight = Weight::Normal,
      "small" => self.size += 4.0,
      "big" => self.size -= 4.0,
      "center" => {
        self.flush();
        self.is_center = false;
      }
      "sup" => {
        self.is_superscript = false;
        self.size *= 2.0;
      }
      "p" => {
        self.flush();
        self.cursor_y += VSTEP;
      }
      "pre" => {
        self.flush();
        self.is_preformatted = false;
      }
      _ => (),
    }
  }
}

fn decode_entities(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();

  while let Some(c) = chars.next() {
    if c != '&' {
      result.push(c);
      continue;
    }

    let mut entity = String::new();
    let mut terminated = false;

    for nc in chars.by_ref() {
      if nc == ';' {
        terminated = true;
        break;
      } else if nc.is_whitespace() {
        entity.push(nc);
        break;
      } else {
        entity.push(nc);
      }
    }

    if terminated {
      let replacement = match entity.as_str() {
        "lt" => Some("<"),
        "gt" => Some(">"),
        "amp" => Some("&"),
        "quot" => Some("\""),
        "apos" => Some("'"),
        "copy" => Some("©"),
        _ => None,
      };

      if let Some(r) = replacement {
        result.push_str(r);
      } else if entity.starts_with('#') {
        let code = if entity.starts_with("#x") || entity.starts_with("#X") {
          u32::from_str_radix(&entity[2..], 16).ok()
        } else {
          entity[1..].parse::<u32>().ok()
        };
        if let Some(n) = code {
          if let Some(ch) = char::from_u32(n) {
            result.push(ch);
          } else {
            result.push('&');
            result.push_str(&entity);
            result.push(';');
          }
        } else {
          result.push('&');
          result.push_str(&entity);
          result.push(';');
        }
      } else {
        result.push('&');
        result.push_str(&entity);
        result.push(';');
      }
    } else {
      result.push('&');
      result.push_str(&entity);
    }
  }

  result
}
