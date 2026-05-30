use crate::runtime::ACTIVE_DOM;

use boa_engine::{Context, JsResult, JsString, JsValue};
use html_parser::{HTMLParser, Node};

use std::cell::RefCell;
use std::rc::Rc;

pub fn js_log(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let msg = args
    .get(0)
    .cloned()
    .unwrap_or_default()
    .display()
    .to_string();

  println!("[JS Log] {}", msg);
  Ok(JsValue::undefined())
}

pub fn js_query_selector_all(
  _: &JsValue,
  args: &[JsValue],
  ctx: &mut Context,
) -> JsResult<JsValue> {
  let selector = args
    .get(0)
    .cloned()
    .unwrap_or_default()
    .display()
    .to_string();
  let mut handles = Vec::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut results = Vec::new();
      find_nodes(&state.tree, &selector, &mut results);

      let mut map = state.handle_map.borrow_mut();
      let mut next = state.next_handle.borrow_mut();

      for node in results {
        let handle = if let Some((&h, _)) = map.iter().find(|(_, n)| Rc::ptr_eq(n, &node)) {
          h
        } else {
          let h = *next;
          *next += 1;
          map.insert(h, node);
          h
        };
        handles.push(handle);
      }
    }
  });

  let array = boa_engine::object::builtins::JsArray::from_iter(
    handles.into_iter().map(|h| JsValue::from(h)),
    ctx,
  );

  Ok(array.into())
}

pub fn js_get_attribute(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let attr = args
    .get(1)
    .cloned()
    .unwrap_or_default()
    .display()
    .to_string();
  let mut result = String::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      if let Some(node) = state.handle_map.borrow().get(&handle) {
        if let Node::Element(e) = &*node.borrow() {
          if let Some(val) = e.attributes.get(&attr) {
            result = val.clone();
          }
        }
      }
    }
  });

  Ok(JsValue::from(JsString::from(result)))
}

pub fn js_inner_html_set(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let html = args
    .get(1)
    .cloned()
    .unwrap_or_default()
    .display()
    .to_string();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      if let Some(node) = state.handle_map.borrow().get(&handle) {
        let full_html = format!("<html><body>{}</body></html>", html);
        let mut parser = HTMLParser::new(full_html);
        let new_doc = parser.parse();

        let new_children = extract_body_children(&new_doc);

        if let Node::Element(e) = &mut *node.borrow_mut() {
          e.children = new_children;
          for child in &e.children {
            if let Node::Element(ce) = &mut *child.borrow_mut() {
              ce.parent = Some(Rc::downgrade(node));
            } else if let Node::Text(ct) = &mut *child.borrow_mut() {
              ct.parent = Some(Rc::downgrade(node));
            }
          }
        }
        *state.needs_relayout.borrow_mut() = true;
      }
    }
  });

  Ok(JsValue::undefined())
}

// Helpers
fn extract_body_children(node: &Rc<RefCell<Node>>) -> Vec<Rc<RefCell<Node>>> {
  let n = node.borrow();
  if let Node::Element(e) = &*n {
    if e.tag == "body" {
      return e.children.clone();
    }
    for c in &e.children {
      let res = extract_body_children(c);
      if !res.is_empty() {
        return res;
      }
    }
  }
  vec![]
}

fn find_nodes(node: &Rc<RefCell<Node>>, selector: &str, results: &mut Vec<Rc<RefCell<Node>>>) {
  let n = node.borrow();
  if let Node::Element(e) = &*n {
    let matches = if selector.starts_with('.') {
      e.attributes
        .get("class")
        .map(|c| c.contains(&selector[1..]))
        .unwrap_or(false)
    } else if selector.starts_with('#') {
      e.attributes
        .get("id")
        .map(|id| id == &selector[1..])
        .unwrap_or(false)
    } else {
      e.tag == selector
    };
    if matches {
      results.push(node.clone());
    }
  }
  for child in n.children() {
    find_nodes(child, selector, results);
  }
}
