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
}

impl Layout {
  pub fn new(tree: &Rc<RefCell<Node>>, width: f32) -> Self {
    let mut layout = Self {
      display_list: DisplayList::new(),
      width: width,
      cursor_x: HSTEP,
      cursor_y: VSTEP,
      line: vec![],
      weight: Weight::Normal,
      style: Style::Normal,
      size: 16.0,
      font_cache: HashMap::new(),
    };

    layout.recurse(tree);
    layout.flush();
    layout
  }

  pub fn recurse(&mut self, node_rc: &Rc<RefCell<Node>>) {
    let node = node_rc.borrow();

    match &*node {
      Node::Text(text) => {
        for word in text.text.split_whitespace() {
          self.word(word.to_string());
        }
      }
      Node::Element(element) => {
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
    };

    let max_ascent = self.line.iter().map(|i| i.size * 0.8).fold(0.0, f32::max);
    let baseline = self.cursor_y + 1.2 * max_ascent;

    for item in &self.line {
      let y = baseline - item.size;

      self
        .display_list
        .add_item(item.x, y, item.word.clone(), item.font, item.size)
    }

    self.cursor_y = baseline + 1.25 * max_ascent;
    self.cursor_x = HSTEP;
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

    if self.cursor_x + word_size.width > self.width - HSTEP {
      self.flush();
    }

    self.line.push(LineItem {
      x: self.cursor_x,
      word,
      font,
      size: self.size,
    });
    self.cursor_x += word_size.width + space_size.width;
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
      "p" => {
        self.flush();
        self.cursor_y += VSTEP;
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
      _ => (),
    }
  }
}
