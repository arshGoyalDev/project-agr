use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub enum Node {
  Text(Text),
  Element(Element),
}

pub struct Text {
  pub text: String,
  pub children: Vec<Rc<RefCell<Node>>>,
  pub parent: Option<Weak<RefCell<Node>>>,
  pub style: HashMap<String, String>,
}

pub struct Element {
  pub tag: String,
  pub self_closing: bool,
  pub children: Vec<Rc<RefCell<Node>>>,
  pub attributes: HashMap<String, String>,
  pub parent: Option<Weak<RefCell<Node>>>,
  pub style: HashMap<String, String>,
}

impl Node {
  pub fn tag(&self) -> Option<&str> {
    match self {
      Node::Element(e) => Some(&e.tag),
      Node::Text(_) => None,
    }
  }

  pub fn children(&self) -> &Vec<Rc<RefCell<Node>>> {
    match self {
      Node::Element(e) => &e.children,
      Node::Text(t) => &t.children,
    }
  }

  pub fn children_mut(&mut self) -> &mut Vec<Rc<RefCell<Node>>> {
    match self {
      Node::Element(e) => &mut e.children,
      Node::Text(t) => &mut t.children,
    }
  }

  pub fn style(&self) -> &HashMap<String, String> {
    match self {
      Node::Element(e) => &e.style,
      Node::Text(t) => &t.style,
    }
  }

  pub fn style_mut(&mut self) -> &mut HashMap<String, String> {
    match self {
      Node::Element(e) => &mut e.style,
      Node::Text(t) => &mut t.style,
    }
  }

  pub fn set_parent(&mut self, p: Weak<RefCell<Node>>) {
    match self {
      Node::Element(e) => e.parent = Some(p),
      Node::Text(t) => t.parent = Some(p),
    }
  }
}
