use crate::parser::{CSSParser, Rule, inherited_properties};
use html_parser::Node as HtmlNode;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn style(node_rc: &Rc<RefCell<HtmlNode>>, rules: &[Rule]) {
  let inherited = inherited_properties();

  // We use this to track the highest priority seen so far for each property
  let mut priority_tracker: HashMap<String, u32> = HashMap::new();

  {
    let mut node = node_rc.borrow_mut();

    let parent_styles: HashMap<String, String> = match &*node {
      HtmlNode::Element(e) => e
        .parent
        .as_ref()
        .and_then(|p| p.upgrade())
        .map(|p| p.borrow().style().clone())
        .unwrap_or_default(),
      HtmlNode::Text(t) => t
        .parent
        .as_ref()
        .and_then(|p| p.upgrade())
        .map(|p| p.borrow().style().clone())
        .unwrap_or_default(),
    };

    for (prop, default_val) in &inherited {
      let val = parent_styles
        .get(*prop)
        .cloned()
        .unwrap_or_else(|| default_val.to_string());
      node.style_mut().insert(prop.to_string(), val);
      priority_tracker.insert(prop.to_string(), 0);
    }

    for rule in rules {
      if rule.selector.matches(&*node) {
        for (prop, prop_val) in &rule.properties {
          let mut effective_priority = rule.priority;
          if prop_val.important {
            effective_priority += 10000;
          }

          let should_insert = match priority_tracker.get(prop) {
            Some(&existing_prio) => effective_priority >= existing_prio,
            None => true,
          };

          if should_insert {
            priority_tracker.insert(prop.clone(), effective_priority);
            node
              .style_mut()
              .insert(prop.clone(), prop_val.value.clone());
          }
        }
      }
    }

    // 4. Apply Inline Styles (Highest base priority: 1000)
    if let HtmlNode::Element(e) = &*node {
      if let Some(inline_style) = e.attributes.get("style") {
        let mut parser = CSSParser::new(inline_style);
        for (prop, prop_val) in parser.body() {
          let mut effective_priority = 1000;
          if prop_val.important {
            effective_priority += 10000;
          }

          let should_insert = match priority_tracker.get(&prop) {
            Some(&existing_prio) => effective_priority >= existing_prio,
            None => true,
          };

          if should_insert {
            priority_tracker.insert(prop.clone(), effective_priority);
            node.style_mut().insert(prop, prop_val.value);
          }
        }
      }
    }

    let font_size = node.style().get("font-size").cloned().unwrap_or_default();
    if font_size.ends_with('%') {
      let parent_font_size = parent_styles
        .get("font-size")
        .cloned()
        .unwrap_or_else(|| inherited["font-size"].to_string());

      if let (Ok(pct), true) = (
        font_size.trim_end_matches('%').parse::<f32>(),
        parent_font_size.ends_with("px"),
      ) {
        if let Ok(parent_px) = parent_font_size.trim_end_matches("px").parse::<f32>() {
          let resolved = (pct / 100.0) * parent_px;
          node
            .style_mut()
            .insert("font-size".to_string(), format!("{}px", resolved));
        }
      }
    }
  }

  let children: Vec<Rc<RefCell<HtmlNode>>> =
    node_rc.borrow().children().iter().map(Rc::clone).collect();
  for child in children {
    style(&child, rules);
  }
}
