use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub enum Node {
  Text(Text),
  Element(Element)
}

pub struct Text {
  pub text: String,
  pub children: Vec<Rc<RefCell<Node>>>,
  pub parent: Option<Weak<RefCell<Node>>>
}

pub struct Element {
  pub tag: String,
  pub children: Vec<Rc<RefCell<Node>>>,
  pub attributes: HashMap<String, String>,
  pub parent: Option<Weak<RefCell<Node>>>
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
}