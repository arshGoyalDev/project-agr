use crate::runtime::ACTIVE_DOM;

use boa_engine::{Context, JsResult, JsString, JsValue};
use html_parser::parser::ParseYield;
use html_parser::{Element, HTMLParser, Node, Text};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn js_log(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let strings: Vec<String> = args.iter().map(|val| js_to_string(Some(val))).collect();
  let msg = strings.join(" ");

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

        let new_doc = loop {
          match parser.resume() {
            ParseYield::Finished(tree) => break tree,
            _ => continue,
          }
        };

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

pub fn js_inner_html_get(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let mut result = String::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      if let Some(node) = state.handle_map.borrow().get(&handle) {
        result = dom_tree_to_html_string(node);
      }
    }
  });

  Ok(JsValue::from(JsString::from(result)))
}

pub fn js_create_element(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let tag = js_to_string(args.get(0));
  let new_node = Rc::new(RefCell::new(Node::Element(Element {
    tag,
    self_closing: false,
    children: vec![],
    attributes: HashMap::new(),
    parent: None,
    style: HashMap::new(),
  })));

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut map = state.handle_map.borrow_mut();
      let mut next = state.next_handle.borrow_mut();
      let h = *next;
      *next += 1;
      map.insert(h, new_node);
      return Ok(JsValue::from(h));
    }
    Ok(JsValue::null())
  })
}

pub fn js_create_text_node(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let text = js_to_string(args.get(0));
  let new_node = Rc::new(RefCell::new(Node::Text(Text {
    text,
    children: vec![],
    parent: None,
    style: HashMap::new(),
  })));

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let mut map = state.handle_map.borrow_mut();
      let mut next = state.next_handle.borrow_mut();
      let h = *next;
      *next += 1;
      map.insert(h, new_node);
      return Ok(JsValue::from(h));
    }
    Ok(JsValue::null())
  })
}

pub fn js_append_child(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let parent_h = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let child_h = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let map = state.handle_map.borrow();
      if let (Some(parent), Some(child)) = (map.get(&parent_h), map.get(&child_h)) {
        parent.borrow_mut().children_mut().push(child.clone());
        child.borrow_mut().set_parent(Rc::downgrade(parent));
        *state.needs_relayout.borrow_mut() = true;
      }
    }
  });
  Ok(JsValue::undefined())
}

pub fn js_insert_before(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let parent_h = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let new_h = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let ref_h = args.get(2).and_then(|v| v.as_number().map(|n| n as usize)); // Can be null/None

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      let map = state.handle_map.borrow();
      if let (Some(parent), Some(new_node)) = (map.get(&parent_h), map.get(&new_h)) {
        let mut parent_borrow = parent.borrow_mut();

        let index = if let Some(ref_h) = ref_h {
          if let Some(ref_node) = map.get(&ref_h) {
            parent_borrow
              .children_mut()
              .iter()
              .position(|c| Rc::ptr_eq(c, ref_node))
          } else {
            None
          }
        } else {
          None
        };

        if let Some(idx) = index {
          parent_borrow.children_mut().insert(idx, new_node.clone());
        } else {
          parent_borrow.children_mut().push(new_node.clone());
        }

        new_node.borrow_mut().set_parent(Rc::downgrade(parent));
        *state.needs_relayout.borrow_mut() = true;
      }
    }
  });
  Ok(JsValue::undefined())
}

pub fn js_text_content_get(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
  let mut result = String::new();

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      if let Some(node) = state.handle_map.borrow().get(&handle) {
        result = text_content_string(node);
      }
    }
  });

  Ok(JsValue::from(JsString::from(result)))
}

pub fn js_text_content_set(_: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
  let handle = args.get(0).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;

  let text_content = js_to_string(args.get(1));

  ACTIVE_DOM.with(|dom| {
    if let Some(state) = &*dom.borrow() {
      if let Some(node) = state.handle_map.borrow_mut().get(&handle) {
        if let Node::Element(e) = &mut *node.borrow_mut() {
          let parent_weak = Rc::downgrade(node);
          let text_node = Rc::new(RefCell::new(Node::Text(Text {
            text: text_content,
            children: vec![],
            parent: Some(parent_weak),
            style: HashMap::new(),
          })));

          e.children = vec![text_node];
        }
        *state.needs_relayout.borrow_mut() = true;
      }
    }
  });

  Ok(JsValue::undefined())
}

// Helpers
fn text_content_string(node: &Rc<RefCell<Node>>) -> String {
  let mut ans = String::new();

  if let Node::Element(elt) = &*node.borrow() {
    for child in &elt.children {
      if let Node::Text(t) = &*child.borrow() {
        ans.push_str(&t.text);
      } else if let Node::Element(_elt) = &*child.borrow() {
        let child_text = text_content_string(child);
        ans.push_str(&child_text);
      }
    }
  }

  return ans;
}

fn dom_tree_to_html_string(node: &Rc<RefCell<Node>>) -> String {
  let mut ans = String::new();

  if let Node::Element(elt) = &*node.borrow() {
    for child in &elt.children {
      if let Node::Element(ch) = &*child.borrow() {
        ans.push('<');
        ans.push_str(&ch.tag);

        for (key, value) in &ch.attributes {
          ans.push_str(&format!(" {}=\"{}\"", key, value));
        }

        if ch.self_closing {
          ans.push_str("/>");
        } else {
          ans.push('>');

          let child_str = dom_tree_to_html_string(child);

          ans.push_str(&child_str);

          ans.push_str("</");
          ans.push_str(&ch.tag);
          ans.push('>');
        }
      } else if let Node::Text(chl) = &*child.borrow() {
        ans.push_str(&chl.text);
      }
    }
  }

  return ans;
}

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
