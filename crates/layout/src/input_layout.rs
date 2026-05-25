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
  // pub fn paint(&self, cmds: &mut DisplayList) {
  //   let node = self.node.borrow();

  //   let bg_color = node
  //     .style()
  //     .get("background-color")
  //     .cloned()
  //     .unwrap_or_else(|| "white".to_string());
  //   let bg_color_val = parse_css_color(&bg_color).unwrap_or(Color::WHITE);

  //   cmds.add_rect(
  //     self.x,
  //     self.y,
  //     self.x + self.width,
  //     self.y + self.height,
  //     bg_color_val,
  //   );

  //   let border_color = node
  //     .style()
  //     .get("border-color")
  //     .cloned()
  //     .unwrap_or_else(|| "gray".to_string());
  //   let border_color_val = parse_css_color(&border_color).unwrap_or(Color::from_rgb(0.5, 0.5, 0.5));

  //   let border_width = node
  //     .style()
  //     .get("border-width")
  //     .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
  //     .unwrap_or(1.0);

  //   cmds.add_rect(
  //     self.x,
  //     self.y,
  //     self.x + self.width,
  //     self.y + border_width,
  //     border_color_val,
  //   );
  //   cmds.add_rect(
  //     self.x,
  //     self.y + self.height - border_width,
  //     self.x + self.width,
  //     self.y + self.height,
  //     border_color_val,
  //   );
  //   cmds.add_rect(
  //     self.x,
  //     self.y,
  //     self.x + border_width,
  //     self.y + self.height,
  //     border_color_val,
  //   );
  //   cmds.add_rect(
  //     self.x + self.width - border_width,
  //     self.y,
  //     self.x + self.width,
  //     self.y + self.height,
  //     border_color_val,
  //   );

  //   let input_type = self.input_type.as_deref().unwrap_or("text");
  //   let is_checked = if let Node::Element(elt) = &*node {
  //     elt.attributes.contains_key("checked")
  //       || elt.attributes.get("checked").map(|s| s.as_str()) == Some("true")
  //   } else {
  //     false
  //   };

  //   if input_type == "checkbox" || input_type == "radio" {
  //     if is_checked {
  //       let margin = self.width.min(self.height) * 0.25;
  //       let check_color = node
  //         .style()
  //         .get("color")
  //         .and_then(|c| parse_css_color(c))
  //         .unwrap_or(Color::BLACK);

  //       cmds.add_rect(
  //         self.x + margin,
  //         self.y + margin,
  //         self.x + self.width - margin,
  //         self.y + self.height - margin,
  //         check_color,
  //       );
  //     }

  //     return;
  //   }

  //   let text = if let Node::Element(e) = &*node {
  //     if e.tag == "input" {
  //       e.attributes.get("value").cloned().unwrap_or_default()
  //     } else if e.tag == "button" {
  //       if let Some(child) = e.children.first() {
  //         if let Node::Text(t) = &*child.borrow() {
  //           t.text.clone()
  //         } else {
  //           String::new()
  //         }
  //       } else {
  //         String::new()
  //       }
  //     } else {
  //       String::new()
  //     }
  //   } else {
  //     String::new()
  //   };

  //   let color = node
  //     .style()
  //     .get("color")
  //     .and_then(|c| crate::block_layout::parse_css_color(c))
  //     .unwrap_or(Color::BLACK);

  //   let is_focused = if let Node::Element(e) = &*node {
  //     e.attributes.get("data-focused").map(|s| s.as_str()) == Some("true")
  //   } else {
  //     false
  //   };

  //   let show_cursor = if let Node::Element(e) = &*node {
  //     e.attributes.get("data-cursor-visible").map(|s| s.as_str()) == Some("true")
  //   } else {
  //     false
  //   };

  //   let mut text_width = 0.0;

  //   if !text.is_empty() {
  //     cmds.add_text(
  //       self.x + 4.0,
  //       self.y + 2.0,
  //       text.clone(),
  //       self.font,
  //       self.size,
  //       color,
  //     );

  //     if is_focused {
  //       let make_paragraph = |content: &str| {
  //         GraphicsParagraph::with_text(AdvancedText {
  //           content,
  //           bounds: Size::INFINITY,
  //           size: Pixels(self.size),
  //           line_height: LineHeight::default(),
  //           font: self.font,
  //           horizontal_alignment: alignment::Horizontal::Left,
  //           vertical_alignment: alignment::Vertical::Top,
  //           shaping: Shaping::Basic,
  //           wrapping: Wrapping::None,
  //         })
  //       };
  //       text_width = make_paragraph(&text).min_bounds().width;
  //     }
  //   }

  //   // DRAW THE BLINKING CURSOR HERE!
  //   if show_cursor {
  //     let cursor_x = self.x + 4.0 + text_width;
  //     cmds.add_rect(
  //       cursor_x,
  //       self.y + 4.0,
  //       cursor_x + 1.0,
  //       self.y + self.height - 4.0,
  //       Color::BLACK,
  //     );
  //   }
  // }

  pub fn paint(&self, cmds: &mut DisplayList) {
    let node = self.node.borrow();

    let bg_color = node
      .style()
      .get("background-color")
      .cloned()
      .unwrap_or_else(|| "white".to_string());
    let bg_color_val = parse_css_color(&bg_color).unwrap_or(Color::WHITE);

    let border_color = node
      .style()
      .get("border-color")
      .cloned()
      .unwrap_or_else(|| "gray".to_string());
    let border_color_val = parse_css_color(&border_color).unwrap_or(Color::from_rgb(0.5, 0.5, 0.5));

    let border_width = node
      .style()
      .get("border-width")
      .and_then(|s| s.trim_end_matches("px").parse::<f32>().ok())
      .unwrap_or(1.0);

    let input_type = self.input_type.as_deref().unwrap_or("text");
    let is_checked = if let Node::Element(e) = &*node {
      e.attributes.contains_key("checked")
        || e.attributes.get("checked").map(|s| s.as_str()) == Some("true")
    } else {
      false
    };

    let check_color = node
      .style()
      .get("color")
      .and_then(|c| crate::block_layout::parse_css_color(c))
      .unwrap_or(Color::BLACK);

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

    // Draw Checkbox (Squares)
    if input_type == "checkbox" {
      cmds.add_rect(
        self.x,
        self.y,
        self.x + self.width,
        self.y + self.height,
        border_color_val,
      ); // Outer border
      cmds.add_rect(
        self.x + border_width,
        self.y + border_width,
        self.x + self.width - border_width,
        self.y + self.height - border_width,
        bg_color_val,
      ); // Inner bg

      if is_checked {
        let margin = self.width.min(self.height) * 0.25;
        cmds.add_rect(
          self.x + margin,
          self.y + margin,
          self.x + self.width - margin,
          self.y + self.height - margin,
          check_color,
        ); // Inner square
      }
      return;
    }

    // Draw Standard Text Inputs
    cmds.add_rect(
      self.x,
      self.y,
      self.x + self.width,
      self.y + self.height,
      bg_color_val,
    );

    // Top, Bottom, Left, Right borders
    cmds.add_rect(
      self.x,
      self.y,
      self.x + self.width,
      self.y + border_width,
      border_color_val,
    );
    cmds.add_rect(
      self.x,
      self.y + self.height - border_width,
      self.x + self.width,
      self.y + self.height,
      border_color_val,
    );
    cmds.add_rect(
      self.x,
      self.y,
      self.x + border_width,
      self.y + self.height,
      border_color_val,
    );
    cmds.add_rect(
      self.x + self.width - border_width,
      self.y,
      self.x + self.width,
      self.y + self.height,
      border_color_val,
    );

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

    let mut text_width = 0.0;
    if !text.is_empty() {
      cmds.add_text(
        self.x + 4.0,
        self.y + 2.0,
        text.clone(),
        self.font,
        self.size,
        check_color,
      );

      if is_focused {
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
        text_width = make_paragraph(&text).min_bounds().width;
      }
    }

    if show_cursor {
      let cursor_x = self.x + 4.0 + text_width;
      cmds.add_rect(
        cursor_x,
        self.y + 4.0,
        cursor_x + 1.0,
        self.y + self.height - 4.0,
        Color::BLACK,
      );
    }
  }

  pub fn get_node(&self, px: f32, py: f32) -> Option<Rc<RefCell<Node>>> {
    if px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height {
      Some(Rc::clone(&self.node))
    } else {
      None
    }
  }
}
