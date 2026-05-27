use crate::block_layout::parse_css_color;
use crate::display_list::DisplayList;
use html_parser::Node;

use iced::Color;
use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::{Paragraph, Text as AdvancedText};
use iced::font::Font;
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Pixels, Size, alignment};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct InputLayout {
  pub node: Rc<RefCell<Node>>,
  pub font: Font,
  pub size: f32,
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub input_type: Option<String>,
}

impl InputLayout {
  pub fn paint(&self, cmds: &mut DisplayList) {
    // Extract all DOM state upfront to avoid MutRef collision panics
    let (
      bg_color_val,
      border_color_val,
      border_width,
      input_type,
      is_checked,
      check_color,
      text,
      is_focused,
      show_cursor,
      pos,
      mut scroll_x,
    ) = {
      let node = self.node.borrow();
      let bg_color = node
        .style()
        .get("background-color")
        .cloned()
        .unwrap_or_else(|| "white".to_string());
      let border_color = node
        .style()
        .get("border-color")
        .cloned()
        .unwrap_or_else(|| "gray".to_string());

      let border_width = node
        .style()
        .get("border-width")
        .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or(1.0);
      let check_color = node
        .style()
        .get("color")
        .and_then(|c| crate::block_layout::parse_css_color(c))
        .unwrap_or(Color::BLACK);

      let input_type = self
        .input_type
        .clone()
        .unwrap_or_else(|| "text".to_string());

      let is_checked = if let Node::Element(e) = &*node {
        e.attributes.contains_key("checked")
          || e.attributes.get("checked").map(|s| s.as_str()) == Some("true")
      } else {
        false
      };

      let text = if let Node::Element(e) = &*node {
        if e.tag == "input" {
          e.attributes.get("value").cloned().unwrap_or_default()
        } else if e.tag == "button" {
          if let Some(child) = e.children.first() {
            if let Node::Text(t) = &*child.borrow() {
              t.text.clone()
            } else {
              String::new()
            }
          } else {
            String::new()
          }
        } else {
          String::new()
        }
      } else {
        String::new()
      };

      let is_focused = if let Node::Element(e) = &*node {
        e.attributes.get("data-focused").map(|s| s.as_str()) == Some("true")
      } else {
        false
      };
      let show_cursor = if let Node::Element(e) = &*node {
        e.attributes.get("data-cursor-visible").map(|s| s.as_str()) == Some("true")
      } else {
        false
      };

      let pos = if let Node::Element(e) = &*node {
        e.attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(text.chars().count())
      } else {
        text.chars().count()
      };

      let scroll_x = if let Node::Element(e) = &*node {
        e.attributes
          .get("data-scroll-x")
          .and_then(|s| s.parse::<f32>().ok())
          .unwrap_or(0.0)
      } else {
        0.0
      };

      (
        parse_css_color(&bg_color).unwrap_or(Color::WHITE),
        parse_css_color(&border_color).unwrap_or(Color::from_rgb(0.5, 0.5, 0.5)),
        border_width,
        input_type,
        is_checked,
        check_color,
        text,
        is_focused,
        show_cursor,
        pos,
        scroll_x,
      )
    };

    // Draw Radio Button (Circles)
    if input_type == "radio" {
      let radius = self.width.min(self.height) / 2.0;
      let cx = self.x + radius;
      let cy = self.y + radius;
      cmds.add_circle(cx, cy, radius, border_color_val);
      cmds.add_circle(cx, cy, radius - border_width, bg_color_val);
      if is_checked {
        cmds.add_circle(cx, cy, radius * 0.5, check_color);
      }
      return;
    }

    // 3. Draw Checkbox (Squares)
    if input_type == "checkbox" {
      cmds.add_rect(
        self.x,
        self.y,
        self.x + self.width,
        self.y + self.height,
        border_color_val,
      );
      cmds.add_rect(
        self.x + border_width,
        self.y + border_width,
        self.x + self.width - border_width,
        self.y + self.height - border_width,
        bg_color_val,
      );
      if is_checked {
        let margin = self.width.min(self.height) * 0.25;
        cmds.add_rect(
          self.x + margin,
          self.y + margin,
          self.x + self.width - margin,
          self.y + self.height - margin,
          check_color,
        );
      }
      return;
    }

    // Draw Standard Text Inputs & Buttons
    cmds.add_rect(
      self.x,
      self.y,
      self.x + self.width,
      self.y + self.height,
      bg_color_val,
    );

    let inner_width = self.width - 8.0; // 4px padding left and right
    let pos = pos.min(text.chars().count());
    let text_chars: Vec<char> = text.chars().collect();

    let make_paragraph = |content: &str| {
      GraphicsParagraph::with_text(AdvancedText {
        content,
        bounds: Size::INFINITY,
        size: Pixels(self.size),
        line_height: LineHeight::default(),
        font: self.font,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
      })
    };

    let text_up_to_cursor: String = text_chars[0..pos].iter().collect();
    let cursor_x_offset = make_paragraph(&text_up_to_cursor).min_bounds().width;
    let total_text_width = make_paragraph(&text).min_bounds().width;

    let max_scroll = (total_text_width - inner_width).max(0.0);

    // Track scroll to keep cursor strictly visible
    if is_focused {
      if cursor_x_offset < scroll_x {
        scroll_x = cursor_x_offset;
      } else if cursor_x_offset > scroll_x + inner_width {
        scroll_x = cursor_x_offset - inner_width;
      }

      scroll_x = scroll_x.clamp(0.0, max_scroll);

      let mut node_mut = self.node.borrow_mut();
      if let Node::Element(e) = &mut *node_mut {
        e.attributes
          .insert("data-scroll-x".to_string(), scroll_x.to_string());
      }
    } else {
      scroll_x = scroll_x.clamp(0.0, max_scroll)
    }

    if !text.is_empty() {
      // Slice the string dynamically so it never draws outside the bounds
      let mut start_idx = 0;
      let mut end_idx = text_chars.len();

      for i in 0..text_chars.len() {
        let s: String = text_chars[0..=i].iter().collect();
        let w = make_paragraph(&s).min_bounds().width;
        if w < scroll_x {
          start_idx = i + 1;
        }
        if w > scroll_x + inner_width {
          end_idx = i; // 1 char overlap so it hides under the border cleanly
          break;
        }
      }
      end_idx = end_idx.min(text_chars.len());

      let visible_string: String = text_chars[start_idx..end_idx].iter().collect();
      let prefix_str: String = text_chars[0..start_idx].iter().collect();
      let offset_x = if start_idx == 0 {
        0.0
      } else {
        make_paragraph(&prefix_str).min_bounds().width
      };

      cmds.add_text(
        self.x + 4.0 + offset_x - scroll_x,
        self.y + 2.0,
        visible_string,
        self.font,
        self.size,
        check_color,
      );
    }

    if show_cursor {
      let cursor_x = self.x + 4.0 + cursor_x_offset - scroll_x;
      if cursor_x >= self.x + 2.0 && cursor_x <= self.x + self.width - 2.0 {
        cmds.add_rect(
          cursor_x,
          self.y + 4.0,
          cursor_x + 1.0,
          self.y + self.height - 4.0,
          Color::BLACK,
        );
      }
    }

    // Paint a tiny scrollbar at the bottom if the text is overflowing
    if total_text_width > inner_width {
      let scroll_ratio = inner_width / total_text_width;
      let scrollbar_width = (inner_width * scroll_ratio).max(10.0);
      let max_scroll = total_text_width - inner_width;
      let scroll_pct = if max_scroll > 0.0 {
        scroll_x / max_scroll
      } else {
        0.0
      };
      let scrollbar_x = self.x + 4.0 + scroll_pct * (inner_width - scrollbar_width);

      cmds.add_rect(
        scrollbar_x,
        self.y + self.height - 4.0,
        scrollbar_x + scrollbar_width,
        self.y + self.height - 2.0,
        Color::from_rgb(0.7, 0.7, 0.7),
      );
    }

    // draw the borders OVER the text. This acts as a visual mask/clip!
    cmds.add_rect(
      self.x,
      self.y,
      self.x + self.width,
      self.y + border_width,
      border_color_val,
    ); // Top
    cmds.add_rect(
      self.x,
      self.y + self.height - border_width,
      self.x + self.width,
      self.y + self.height,
      border_color_val,
    ); // Bottom
    cmds.add_rect(
      self.x,
      self.y,
      self.x + border_width,
      self.y + self.height,
      border_color_val,
    ); // Left
    cmds.add_rect(
      self.x + self.width - border_width,
      self.y,
      self.x + self.width,
      self.y + self.height,
      border_color_val,
    ); // Right
  }

  pub fn get_node(&self, px: f32, py: f32) -> Option<Rc<RefCell<Node>>> {
    if px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height {
      Some(Rc::clone(&self.node))
    } else {
      None
    }
  }
}
