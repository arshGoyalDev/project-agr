use crate::block_layout::BlockLayout;
use crate::display_list::DisplayList;
use crate::document_layout::DocumentLayout;
use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

pub const HSTEP: f32 = 9.0;
pub const VSTEP: f32 = 15.0;
pub const PRE_BG: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

pub fn paint_tree(layout: &BlockLayout, display_list: &mut DisplayList) {
  display_list.extend(&layout.paint());
  for child in &layout.children {
    paint_tree(child, display_list);
  }
}

pub fn paint_tree_document(doc: &DocumentLayout, display_list: &mut DisplayList) {
  // DocumentLayout.paint() returns nothing, go straight to children
  for child in &doc.children {
    paint_tree(child, display_list);
  }
}

pub fn decode_entities(text: &str) -> String {
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

pub struct Layout {
  pub display_list: DisplayList,
  pub height: f32,
}

impl Layout {
  pub fn new(tree: &Rc<RefCell<Node>>, width: f32) -> Self {
    let mut doc = DocumentLayout::new(tree);
    doc.layout(width);

    let mut display_list = DisplayList::new();
    paint_tree_document(&doc, &mut display_list);

    Self {
      height: doc.height,
      display_list,
    }
  }
}
