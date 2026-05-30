use html_parser::Node;
use js_bindings::JsRuntime;
use layout::{DisplayList, DocumentLayout};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
  pub url: String,
  pub payload: Option<String>,
}

pub struct Tab {
  pub url: String,
  pub history: Vec<HistoryEntry>,
  pub history_index: usize,
  pub tree: Option<Rc<RefCell<Node>>>,
  pub document: Option<DocumentLayout>,
  pub display_list: DisplayList,
  pub scroll_offset: f32,
  pub max_y: f32,
  pub title: String,
  pub focus: Option<Rc<RefCell<Node>>>,
  pub js_runtime: JsRuntime,
}

impl Tab {
  pub fn new(url: String) -> Self {
    Self {
      url: url.clone(),
      history: vec![HistoryEntry {
        url: url.clone(),
        payload: None,
      }],
      history_index: 0,
      tree: None,
      document: None,
      display_list: DisplayList::new(),
      scroll_offset: 0.0,
      max_y: 0.0,
      title: String::new(),
      focus: None,
      js_runtime: JsRuntime::new(),
    }
  }

  pub fn keypress(&mut self, c: char) {
    if let Some(focused) = &self.focus {
      let mut node = focused.borrow_mut();
      if let Node::Element(elt) = &mut *node {
        let value_str = elt.attributes.get("value").cloned().unwrap_or_default();
        let mut pos = elt
          .attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(value_str.chars().count());

        let mut chars: Vec<char> = value_str.chars().collect();
        pos = pos.min(chars.len());
        chars.insert(pos, c);

        elt
          .attributes
          .insert("value".to_string(), chars.into_iter().collect());
        elt
          .attributes
          .insert("data-cursor-pos".to_string(), (pos + 1).to_string());
      }
    }
  }

  pub fn blur(&mut self) -> bool {
    if let Some(focused) = self.focus.take() {
      let mut node = focused.borrow_mut();

      if let Node::Element(elt) = &mut *node {
        elt.attributes.remove("data-focused");
        elt.attributes.remove("data-cursor-visible");
      }
      true
    } else {
      false
    }
  }

  pub fn backspace(&mut self) {
    if let Some(focused) = &self.focus {
      let mut node = focused.borrow_mut();
      if let Node::Element(elt) = &mut *node {
        let value_str = elt.attributes.get("value").cloned().unwrap_or_default();
        let mut pos = elt
          .attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(value_str.chars().count());

        if pos > 0 {
          let mut chars: Vec<char> = value_str.chars().collect();
          pos = pos.min(chars.len());
          chars.remove(pos - 1);

          elt
            .attributes
            .insert("value".to_string(), chars.into_iter().collect());
          elt
            .attributes
            .insert("data-cursor-pos".to_string(), (pos - 1).to_string());
        }
      }
    }
  }

  pub fn delete(&mut self) {
    if let Some(focused) = &self.focus {
      let mut node = focused.borrow_mut();
      if let Node::Element(elt) = &mut *node {
        let value_str = elt.attributes.get("value").cloned().unwrap_or_default();
        let pos = elt
          .attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(value_str.chars().count());

        let mut chars: Vec<char> = value_str.chars().collect();

        if pos < chars.len() {
          chars.remove(pos);

          elt
            .attributes
            .insert("value".to_string(), chars.into_iter().collect());

          elt
            .attributes
            .insert("data-cursor-pos".to_string(), pos.to_string());
        }
      }
    }
  }

  pub fn arrow_left(&mut self) {
    if let Some(focused) = &self.focus {
      let mut node = focused.borrow_mut();
      if let Node::Element(elt) = &mut *node {
        let value_str = elt.attributes.get("value").cloned().unwrap_or_default();
        let pos = elt
          .attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(value_str.chars().count());
        if pos > 0 {
          elt
            .attributes
            .insert("data-cursor-pos".to_string(), (pos - 1).to_string());
        }
      }
    }
  }

  pub fn arrow_right(&mut self) {
    if let Some(focused) = &self.focus {
      let mut node = focused.borrow_mut();
      if let Node::Element(elt) = &mut *node {
        let value_str = elt.attributes.get("value").cloned().unwrap_or_default();
        let pos = elt
          .attributes
          .get("data-cursor-pos")
          .and_then(|s| s.parse::<usize>().ok())
          .unwrap_or(value_str.chars().count());
        if pos < value_str.chars().count() {
          elt
            .attributes
            .insert("data-cursor-pos".to_string(), (pos + 1).to_string());
        }
      }
    }
  }

  pub fn submit_form(&mut self, start_node: Rc<RefCell<Node>>) -> Option<(String, String, String)> {
    let mut current = Some(start_node);
    let mut form_action = None;
    let mut form_method = None;
    let mut form_node = None;

    while let Some(node_rc) = current {
      let node = node_rc.borrow();

      let is_form = {
        if let Node::Element(elt) = &*node {
          if elt.tag == "form" {
            form_action = elt.attributes.get("action").cloned();
            form_method = Some(
              elt
                .attributes
                .get("method")
                .cloned()
                .unwrap_or_else(|| "GET".to_string()),
            );
            true
          } else {
            false
          }
        } else {
          false
        }
      };

      if is_form {
        form_node = Some(node_rc.clone());
        break;
      }

      current = {
        let node = node_rc.borrow();
        match &*node {
          Node::Element(elt) => elt.parent.as_ref().and_then(|w| w.upgrade()),
          Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
        }
      };
    }

    if let Some(action) = form_action {
      let mut payload = String::new();

      if let Some(form) = form_node {
        if !self.js_runtime.dispatch_event("submit", form.clone()) {
          return None;
        }

        let mut inputs = Vec::new();
        find_inputs(&form, &mut inputs);

        for (name, value) in inputs {
          if !payload.is_empty() {
            payload.push('&');
          }

          payload.push_str(&format!(
            "{}={}",
            name.replace(" ", "+"),
            value.replace(' ', "+")
          ));
        }
      }

      let method = form_method.unwrap().to_uppercase();
      return Some((action, payload, method));
    }

    None
  }
}

fn find_inputs(node_rc: &Rc<RefCell<Node>>, inputs: &mut Vec<(String, String)>) {
  let node = node_rc.borrow();

  if let Node::Element(elt) = &*node {
    if elt.tag == "input" {
      let input_type = elt
        .attributes
        .get("type")
        .map(|s| s.as_str())
        .unwrap_or("text");

      if input_type == "checkbox" || input_type == "radio" {
        if elt.attributes.contains_key("checked")
          || elt.attributes.get("checked").map(|s| s.as_str()) == Some("true")
        {
          if let Some(name) = elt.attributes.get("name") {
            let value = elt
              .attributes
              .get("value")
              .cloned()
              .unwrap_or_else(|| "on".to_string());
            inputs.push((name.clone(), value));
          }
        }
      } else {
        if let Some(name) = elt.attributes.get("name") {
          let value = elt.attributes.get("value").cloned().unwrap_or_default();
          inputs.push((name.clone(), value));
        }
      }
    }
  }

  for child in node.children() {
    find_inputs(child, inputs);
  }
}
