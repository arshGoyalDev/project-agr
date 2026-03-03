use crate::utils::Node;
use std::cell::RefCell;
use std::rc::Rc;

pub fn syntax_highlight(node: &Rc<RefCell<Node>>) -> String {
  let mut result = String::from("<pre>");
  walk(node, &mut result, 0);
  result.push_str("</pre>");
  result
}

fn walk(node: &Rc<RefCell<Node>>, out: &mut String, depth: usize) {
  let borrowed = node.borrow();

  match &*borrowed {
    crate::utils::Node::Text(t) => {
      let trimmed = t.text.trim();
      if trimmed.is_empty() {
        return;
      }
      out.push_str(&indent(depth));
      out.push_str("<b>");
      out.push_str(&escape_html(trimmed));
      out.push_str("</b>");
      out.push('\n');
    }
    crate::utils::Node::Element(e) => {
      out.push_str(&indent(depth));
      out.push_str(&escape_html(&format_open_tag(e)));
      out.push('\n');

      for child in &e.children {
        walk(child, out, depth + 4);
      }

      if !e.children.is_empty() {
        out.push_str(&indent(depth));
        out.push_str(&escape_html(&format!("</{}>", e.tag)));
        out.push('\n');
      }
    }
  }
}

fn indent(depth: usize) -> String {
  " ".repeat(depth)
}

fn format_open_tag(e: &crate::utils::Element) -> String {
  let mut s = format!("<{}", e.tag);

  let mut attrs: Vec<(&String, &String)> = e.attributes.iter().collect();
  attrs.sort_by_key(|(k, _)| k.as_str());

  for (key, value) in attrs {
    if value.is_empty() {
      s.push_str(&format!(" {}", key));
    } else {
      s.push_str(&format!(" {}=\"{}\"", key, value));
    }
  }
  s.push('>');
  s
}

fn escape_html(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
}
