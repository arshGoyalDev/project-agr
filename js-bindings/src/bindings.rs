use crate::runtime::ACTIVE_DOM;

use boa_engine::{Context, JsResult, JsString, JsValue};
use html_parser::{HTMLParser, Node};

use std::cell::RefCell;
use std::rc::Rc;

pub fn js_log(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let msg = js_to_string(args.get(0));

  println!("[JS Log] {}", msg);
  Ok(JsValue::undefined())
}

pub fn js_query_selector_all(
  _: &JsValue,
  args: &[JsValue],
  ctx: &mut Context,
) -> JsResult<JsValue> {
  let selector = js_to_string(args.get(0));
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

pub fn js_query_selector(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let selector = js_to_string(args.get(0));
  let mut handle = None;

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut results = Vec::new();
      find_nodes(&state.tree, &selector, &mut results);

      let node = results[0].clone();

      let mut map = state.handle_map.borrow_mut();
      let mut next = state.next_handle.borrow_mut();

      let node_handle = if let Some((&h, _)) = map.iter().find(|(_, n)| Rc::ptr_eq(n, &node)) {
        h
      } else {
        let h = *next;
        *next += 1;
        map.insert(h, node);
        h
      };
      handle = Some(node_handle);
    }
  });

  if let Some(handle_node) = handle {
    Ok(JsValue::from(handle_node))
  } else {
    Ok(JsValue::null())
  }
}

pub fn js_node_children(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;

  let mut handles = Vec::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let target_node_opt = state.handle_map.borrow().get(&handle).cloned();

      if let Some(target_node) = target_node_opt {
        let mut results = Vec::new();
        find_children(&target_node, &mut results);

        let mut map = state.handle_map.borrow_mut();
        let mut next = state.next_handle.borrow_mut();

        for node in results {
          let child_handle = if let Some((&h, _)) = map.iter().find(|(_, n)| Rc::ptr_eq(n, &node)) {
            h
          } else {
            let h = *next;
            *next += 1;
            map.insert(h, node);
            h
          };
          handles.push(child_handle);
        }
      }
    }
  });

  let array = boa_engine::object::builtins::JsArray::from_iter(
    handles.into_iter().map(|h| JsValue::from(h)),
    ctx,
  );
  Ok(array.into())
}

pub fn js_get_element_by_id(
  _: &JsValue,
  args: &[JsValue],
  _ctx: &mut Context,
) -> JsResult<JsValue> {
  let id = js_to_string(args.get(0));
  let selector = format!("#{}", id);
  let mut handle = None;

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut results = Vec::new();
      find_nodes(&state.tree, &selector, &mut results);

      if !results.is_empty() {
        let node = results[0].clone();
        let mut map = state.handle_map.borrow_mut();
        let mut next = state.next_handle.borrow_mut();

        let node_handle = if let Some((&h, _)) = map.iter().find(|(_, n)| Rc::ptr_eq(n, &node)) {
          h
        } else {
          let h = *next;
          *next += 1;
          map.insert(h, node);
          h
        };
        handle = Some(node_handle);
      }
    }
  });

  if let Some(h) = handle {
    Ok(JsValue::from(h))
  } else {
    Ok(JsValue::null())
  }
}

pub fn js_get_elements_by_class_name(
  _: &JsValue,
  args: &[JsValue],
  ctx: &mut Context,
) -> JsResult<JsValue> {
  let class_name = js_to_string(args.get(0));
  let selector = format!(".{}", class_name);
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

pub fn js_get_elements_by_tag_name(
  _: &JsValue,
  args: &[JsValue],
  ctx: &mut Context,
) -> JsResult<JsValue> {
  let tag_name = js_to_string(args.get(0)).to_lowercase();
  let mut handles = Vec::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut results = Vec::new();
      find_nodes(&state.tree, &tag_name, &mut results);

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

  let attr = js_to_string(args.get(1));
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

  let html = js_to_string(args.get(1));

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

fn js_to_string(val: Option<&JsValue>) -> String {
  if let Some(v) = val {
    if let Some(s) = v.as_string() {
      return s.to_std_string_escaped();
    }
    v.display().to_string()
  } else {
    String::new()
  }
}

fn find_children(node: &Rc<RefCell<Node>>, results: &mut Vec<Rc<RefCell<Node>>>) {
  let n = node.borrow();

  if let Node::Element(e) = &*n {
    for child in &e.children {
      if let Node::Element(_) = &*child.borrow() {
        results.push(Rc::clone(child));
      }
    }
  }
}
