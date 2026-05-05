use html_parser::Node;

pub trait Selector {
  fn matches(&self, node: &Node) -> bool;
  fn priority(&self) -> u32;
}

pub struct TagSelector {
  pub tag: String,
}

impl Selector for TagSelector {
  fn matches(&self, node: &Node) -> bool {
    match node {
      Node::Element(e) => e.tag == self.tag,
      _ => false,
    }
  }

  fn priority(&self) -> u32 {
    1
  }
}

pub struct DescendantSelector {
  pub ancestor: Box<dyn Selector>,
  pub descendant: Box<dyn Selector>,
  priority: u32,
}

impl DescendantSelector {
  pub fn new(ancestor: Box<dyn Selector>, descendant: Box<dyn Selector>) -> Self {
    let priority = ancestor.priority() + descendant.priority();
    Self {
      ancestor,
      descendant,
      priority,
    }
  }
}

impl Selector for DescendantSelector {
  fn matches(&self, node: &Node) -> bool {
    if !self.descendant.matches(node) {
      return false;
    }

    // Walk up the parent chain
    let mut current = match node {
      Node::Element(e) => e.parent.as_ref().and_then(|w| w.upgrade()),
      Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
    };
    while let Some(parent_rc) = current {
      let parent = parent_rc.borrow();
      if self.ancestor.matches(&*parent) {
        return true;
      }
      current = match &*parent {
        Node::Element(e) => e.parent.as_ref().and_then(|w| w.upgrade()),
        Node::Text(t) => t.parent.as_ref().and_then(|w| w.upgrade()),
      };
    }
    false
  }

  fn priority(&self) -> u32 {
    self.priority
  }
}
