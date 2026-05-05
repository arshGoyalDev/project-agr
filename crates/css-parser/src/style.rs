use crate::parser::{CSSParser, Rule, inherited_properties};
use html_parser::Node as HtmlNode;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn style(node_rc: &Rc<RefCell<HtmlNode>>, rules: &[Rule]) {
  let inherited = inherited_properties();

  {
    let mut node = node_rc.borrow_mut();

    let parent_styles: HashMap<String, String> = match &*node {
      HtmlNode::Element(e) => {
        if let Some(parent_weak) = &e.parent {
          if let Some(parent_rc) = parent_weak.upgrade() {
            let parent = parent_rc.borrow();
            parent.style().clone()
          } else {
            HashMap::new()
          }
        } else {
          HashMap::new()
        }
      }
      HtmlNode::Text(t) => {
        if let Some(parent_weak) = &t.parent {
          if let Some(parent_rc) = parent_weak.upgrade() {
            let parent = parent_rc.borrow();
            parent.style().clone()
          } else {
            HashMap::new()
          }
        } else {
          HashMap::new()
        }
      }
    };

    for (prop, default_val) in &inherited {
      if let Some(parent_val) = parent_styles.get(*prop) {
        node
          .style_mut()
          .insert(prop.to_string(), parent_val.clone());
      } else {
        node
          .style_mut()
          .insert(prop.to_string(), default_val.to_string());
      }
    }

    for rule in rules {
      if rule.selector.matches(&*node) {
        for (prop, val) in &rule.properties {
          node.style_mut().insert(prop.clone(), val.clone());
        }
      }
    }

    if let HtmlNode::Element(e) = &*node {
      if let Some(inline_style) = e.attributes.get("style") {
        let mut parser = CSSParser::new(inline_style);
        for (prop, val) in parser.body() {
          node.style_mut().insert(prop, val);
        }
      }
    }

    let font_size = node
      .style_mut()
      .get("font-size")
      .cloned()
      .unwrap_or_default();
    if font_size.ends_with('%') {
      let parent_font_size = if !parent_styles.is_empty() {
        parent_styles
          .get("font-size")
          .cloned()
          .unwrap_or_else(|| inherited["font-size"].to_string())
      } else {
        inherited["font-size"].to_string()
      };

      if let (Ok(pct), true) = (
        font_size.trim_end_matches('%').parse::<f32>(),
        parent_font_size.ends_with("px"),
      ) {
        if let Ok(parent_px) = parent_font_size.trim_end_matches("px").parse::<f32>() {
          let resolved = pct / 100.0 * parent_px;
          node
            .style_mut()
            .insert("font-size".to_string(), format!("{}px", resolved));
        }
      }
    }
  }

  let children: Vec<Rc<RefCell<HtmlNode>>> = {
    let node = node_rc.borrow();
    node.children().iter().map(|c| Rc::clone(c)).collect()
  };

  for child in &children {
    style(child, rules);
  }
}
