use crate::bindings::{
  js_get_attribute, js_get_element_by_id, js_get_elements_by_class_name,
  js_get_elements_by_tag_name, js_inner_html_get, js_inner_html_set, js_log, js_node_children,
  js_query_selector, js_query_selector_all,
};

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsString, NativeFunction, Source};
use html_parser::Node;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct DomState {
  pub tree: Rc<RefCell<Node>>,
  pub handle_map: Rc<RefCell<HashMap<usize, Rc<RefCell<Node>>>>>,
  pub next_handle: Rc<RefCell<usize>>,
  pub needs_relayout: Rc<RefCell<bool>>,
}

thread_local! {
  pub static ACTIVE_DOM: RefCell<Option<DomState>> = RefCell::new(None);
}

pub struct JsRuntime {
  context: Context,
  state: Option<DomState>,
}

impl JsRuntime {
  pub fn new() -> Self {
    let mut context = Context::default();

    let rust_obj = ObjectInitializer::new(&mut context)
      .function(
        NativeFunction::from_fn_ptr(js_log),
        JsString::from("log"),
        1,
      )
      .function(
        NativeFunction::from_fn_ptr(js_query_selector_all),
        JsString::from("querySelectorAll"),
        1,
      )
      .function(
        NativeFunction::from_fn_ptr(js_query_selector),
        JsString::from("querySelector"),
        1,
      )
      .function(
        NativeFunction::from_fn_ptr(js_get_attribute),
        JsString::from("getAttribute"),
        2,
      )
      .function(
        NativeFunction::from_fn_ptr(js_inner_html_set),
        JsString::from("innerHTML_set"),
        2,
      )
      .function(
        NativeFunction::from_fn_ptr(js_inner_html_get),
        JsString::from("innerHTML_get"),
        2,
      )
      .function(
        NativeFunction::from_fn_ptr(js_node_children),
        JsString::from("node_children"),
        2,
      )
      .function(
        NativeFunction::from_fn_ptr(js_get_element_by_id),
        JsString::from("getElementById"),
        1,
      )
      .function(
        NativeFunction::from_fn_ptr(js_get_elements_by_class_name),
        JsString::from("getElementsByClassName"),
        1,
      )
      .function(
        NativeFunction::from_fn_ptr(js_get_elements_by_tag_name),
        JsString::from("getElementsByTagName"),
        1,
      )
      .build();

    let _ =
      context.register_global_property(JsString::from("__rust__"), rust_obj, Attribute::all());
    let runtime_js = include_str!("../../assets/runtime.js");
    let _ = context.eval(Source::from_bytes(runtime_js));

    Self {
      context,
      state: None,
    }
  }

  pub fn set_dom_tree(&mut self, tree: Rc<RefCell<Node>>) {
    self.state = Some(DomState {
      tree,
      handle_map: Rc::new(RefCell::new(HashMap::new())),
      next_handle: Rc::new(RefCell::new(0)),
      needs_relayout: Rc::new(RefCell::new(false)),
    });
  }

  pub fn run(&mut self, code: &str) -> bool {
    if let Some(state) = &self.state {
      ACTIVE_DOM.with(|d| *d.borrow_mut() = Some(state.clone()));
      let res = self.context.eval(Source::from_bytes(code));
      ACTIVE_DOM.with(|d| *d.borrow_mut() = None);

      if let Err(error) = res {
        println!("[JS Error] {}", error);
      }

      let needs_relayout = *state.needs_relayout.borrow();
      *state.needs_relayout.borrow_mut() = false;

      needs_relayout
    } else {
      false
    }
  }

  pub fn dispatch_event(&mut self, event_type: &str, node: Rc<RefCell<Node>>) -> bool {
    if let Some(state) = &self.state {
      ACTIVE_DOM.with(|d| *d.borrow_mut() = Some(state.clone()));

      let handle = {
        let mut map = state.handle_map.borrow_mut();
        let mut next = state.next_handle.borrow_mut();

        if let Some((&h, _)) = map.iter().find(|(_, n)| Rc::ptr_eq(n, &node)) {
          h
        } else {
          let h = *next;
          *next += 1;
          map.insert(h, node.clone());
          h
        }
      };

      let script = format!(
        "new Node({}).dispatchEvent(new Event('{}'))",
        handle, event_type
      );
      let res = self.context.eval(Source::from_bytes(&script));
      ACTIVE_DOM.with(|d| *d.borrow_mut() = None);

      match res {
        Ok(val) => val.as_boolean().unwrap_or(true),
        Err(error) => {
          println!("[JS Error in dispatch_event] {}", error);
          true
        }
      }
    } else {
      true
    }
  }
}
