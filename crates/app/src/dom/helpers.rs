use html_parser::Node;

use std::cell::RefCell;
use std::rc::Rc;

use iced::Color;

/// Walks the DOM tree and collects all `<link rel="stylesheet" href="...">` hrefs.
pub fn find_stylesheet_links(node_rc: &Rc<RefCell<Node>>, links: &mut Vec<String>) {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "link" {
      if let Some(rel) = e.attributes.get("rel") {
        if rel == "stylesheet" {
          if let Some(href) = e.attributes.get("href") {
            links.push(href.clone());
          }
        }
      }
    }
  }

  for child in node.children() {
    find_stylesheet_links(child, links);
  }
}

/// Walks the DOM tree and collects the text content of all `<style>` elements.
pub fn find_inline_styles(node_rc: &Rc<RefCell<Node>>, inline_rules: &mut Vec<String>) {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "style" {
      for child_rc in &e.children {
        let child = child_rc.borrow();
        if let Node::Text(t) = &*child {
          inline_rules.push(t.text.clone());
        }
      }
    }
  }

  for child in node.children() {
    find_inline_styles(child, inline_rules);
  }
}

/// Returns the trimmed text content of the first `<title>` element found, if any.
pub fn extract_title(node_rc: &Rc<RefCell<Node>>) -> Option<String> {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "title" {
      for child_rc in &e.children {
        if let Node::Text(t) = &*child_rc.borrow() {
          let trimmed = t.text.trim();
          if !trimmed.is_empty() {
            return Some(trimmed.to_string());
          }
        }
      }
    }
  }

  for child in node.children() {
    if let Some(title) = extract_title(child) {
      return Some(title);
    }
  }

  None
}

/// Walks the DOM looking for a `background-color` style on `<html>` or `<body>`.
pub fn get_page_bg_color(node_rc: &Rc<RefCell<Node>>) -> Option<iced::Color> {
  let node = node_rc.borrow();

  if let Node::Element(e) = &*node {
    if e.tag == "html" || e.tag == "body" {
      if let Some(bgcolor) = node.style().get("background-color") {
        if bgcolor != "transparent" {
          if let Some(color) = parse_css_color(bgcolor) {
            return Some(color);
          }
        }
      }
    }
  }

  for child in node.children() {
    if let Some(color) = get_page_bg_color(child) {
      return Some(color);
    }
  }

  None
}

/// Parses a CSS color string (named colors, `#rrggbb`, `#rgb`) into an `iced::Color`.
pub fn parse_css_color(s: &str) -> Option<iced::Color> {
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

  // Handle rgb(r, g, b)
  if s.starts_with("rgb(") && s.ends_with(')') {
    let inner = &s[4..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 3 {
      let r = parts[0].trim().parse::<u8>().ok()?;
      let g = parts[1].trim().parse::<u8>().ok()?;
      let b = parts[2].trim().parse::<u8>().ok()?;
      return Some(Color::from_rgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
      ));
    }
  }

  // Handle rgba(r, g, b, a)
  if s.starts_with("rgba(") && s.ends_with(')') {
    let inner = &s[5..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 4 {
      let r = parts[0].trim().parse::<u8>().ok()?;
      let g = parts[1].trim().parse::<u8>().ok()?;
      let b = parts[2].trim().parse::<u8>().ok()?;
      let a = parts[3].trim().parse::<f32>().ok()?;

      return Some(Color::from_rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a,
      ));
    }
  }

  // Handle Hex colors (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
  if s.starts_with('#') {
    let hex = &s[1..];
    match hex.len() {
      3 => {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        return Some(Color::from_rgb(
          r as f32 / 255.0,
          g as f32 / 255.0,
          b as f32 / 255.0,
        ));
      }
      4 => {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
        return Some(Color::from_rgba(
          r as f32 / 255.0,
          g as f32 / 255.0,
          b as f32 / 255.0,
          a as f32 / 255.0,
        ));
      }
      6 => {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some(Color::from_rgb(
          r as f32 / 255.0,
          g as f32 / 255.0,
          b as f32 / 255.0,
        ));
      }
      8 => {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        return Some(Color::from_rgba(
          r as f32 / 255.0,
          g as f32 / 255.0,
          b as f32 / 255.0,
          a as f32 / 255.0,
        ));
      }
      _ => {}
    }
  }

  // hsl(h, s%, l%)
  if s.starts_with("hsl(") && s.ends_with(')') {
    let inner = &s[4..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 3 {
      let h = parts[0]
        .trim()
        .trim_end_matches("deg")
        .parse::<f32>()
        .ok()?;

      let s_val = parts[1].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
      let l_val = parts[2].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;

      let (r, g, b) = hsl_to_rgb(h, s_val, l_val);
      return Some(Color::from_rgb(r, g, b));
    }
  }

  // hsla(h, s%, l%, a)
  if s.starts_with("hsla(") && s.ends_with(')') {
    let inner = &s[5..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() == 4 {
      let h = parts[0]
        .trim()
        .trim_end_matches("deg")
        .parse::<f32>()
        .ok()?;
      let s_val = parts[1].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
      let l_val = parts[2].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
      let a = parts[3].trim().parse::<f32>().ok()?; // Alpha is standard float

      let (r, g, b) = hsl_to_rgb(h, s_val, l_val);
      return Some(Color::from_rgba(r, g, b, a));
    }
  }

  None
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
  let h_normalized = (h % 360.0 + 360.0) % 360.0 / 360.0;
  let s_clamped = s.clamp(0.0, 1.0);
  let l_clamped = l.clamp(0.0, 1.0);

  if s_clamped == 0.0 {
    return (l_clamped, l_clamped, l_clamped);
  }

  let q = if l_clamped < 0.5 {
    l_clamped * (1.0 + s_clamped)
  } else {
    l_clamped + s_clamped - l_clamped * s_clamped
  };
  let p = 2.0 * l_clamped - q;

  let hue_to_rgb = |mut t: f32| -> f32 {
    if t < 0.0 {
      t += 1.0;
    }
    if t > 1.0 {
      t -= 1.0;
    }
    if t < 1.0 / 6.0 {
      return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
      return q;
    }
    if t < 2.0 / 3.0 {
      return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
  };

  (
    hue_to_rgb(h_normalized + 1.0 / 3.0),
    hue_to_rgb(h_normalized),
    hue_to_rgb(h_normalized - 1.0 / 3.0),
  )
}
