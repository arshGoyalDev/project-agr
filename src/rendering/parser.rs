use crate::utils::{Element, Node, Text};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const SELF_CLOSING_TAGS: [&str; 14] = [
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
  "track", "wbr",
];

const HEAD_TAGS: [&str; 9] = [
  "base", "basefont", "bgsound", "noscript", "link", "meta", "title", "style", "script",
];

pub struct HTMLParser {
  body: String,
  unfinished: Vec<Rc<RefCell<Node>>>,
}

impl HTMLParser {
  pub fn new(body: String) -> Self {
    HTMLParser {
      body: body,
      unfinished: vec![],
    }
  }

  pub fn parse(&mut self) -> Rc<RefCell<Node>> {
    let body = self.body.clone();
    let mut text = String::new();
    let mut in_tag = false;

    for c in body.chars() {
      if c == '<' {
        in_tag = true;
        if !text.is_empty() {
          self.add_text(text.clone());
        }
        text.clear()
      } else if c == '>' {
        in_tag = false;
        self.add_tag(text.clone());
        text.clear()
      } else {
        text.push(c)
      }
    }

    if !in_tag && !text.is_empty() {
      self.add_text(text);
    }

    self.finish()
  }

  fn add_text(&mut self, text: String) {
    if text.trim().is_empty() {
      return;
    }

    self.implicit_tags(None);

    let parent_rc = self.unfinished.last().unwrap().clone();
    let parent_weak = Rc::downgrade(&parent_rc);

    let node = Rc::new(RefCell::new(Node::Text(Text {
      text,
      parent: Some(parent_weak),
      children: vec![],
    })));

    parent_rc.borrow_mut().children_mut().push(node);
  }

  fn add_tag(&mut self, tag: String) {
    let (tag, attributes) = self.get_attributes(&tag);

    if tag.starts_with('!') {
      return;
    }

    self.implicit_tags(Some(&tag.clone()));

    if tag.starts_with('/') {
      if self.unfinished.len() == 1 {
        return;
      }

      let node = self.unfinished.pop().unwrap();
      let parent_rc = self.unfinished.last().unwrap().clone();

      parent_rc.borrow_mut().children_mut().push(node);
    } else if SELF_CLOSING_TAGS.contains(&tag.as_str()) {
      let parent_rc = self.unfinished.last().unwrap().clone();
      let parent_weak = Rc::downgrade(&parent_rc);

      let node = Rc::new(RefCell::new(Node::Element(Element {
        tag,
        attributes,
        parent: Some(parent_weak),
        children: vec![],
      })));

      parent_rc.borrow_mut().children_mut().push(node);
    } else {
      let parent_weak = self.unfinished.last().map(|p| Rc::downgrade(p));

      let node = Rc::new(RefCell::new(Node::Element(Element {
        tag,
        attributes,
        parent: parent_weak,
        children: vec![],
      })));

      self.unfinished.push(node)
    }
  }

  fn get_attributes(&self, text: &str) -> (String, HashMap<String, String>) {
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.is_empty() {
      return (String::new(), HashMap::new());
    }

    let tag = parts[0].to_lowercase();
    let mut attributes = HashMap::new();

    for attrpair in &parts[1..] {
      if let Some(pos) = attrpair.find('=') {
        let key = attrpair[..pos].to_lowercase();
        let mut value = attrpair[pos + 1..].to_string();

        if value.len() > 2 && (value.starts_with('"') || value.starts_with('\'')) {
          value = value[1..value.len() - 1].to_string();
        }

        attributes.insert(key, value);
      } else {
        attributes.insert(attrpair.to_lowercase(), String::new());
      }
    }

    (tag, attributes)
  }

  fn finish(&mut self) -> Rc<RefCell<Node>> {
    if self.unfinished.is_empty() {
      self.implicit_tags(None);
    }

    while self.unfinished.len() > 1 {
      let node = self.unfinished.pop().unwrap();
      let parent_rc = self.unfinished.last().unwrap().clone();
      parent_rc.borrow_mut().children_mut().push(node);
    }

    self.unfinished.pop().unwrap()
  }

  fn implicit_tags(&mut self, tag: Option<&str>) {
    loop {
      let open_tags: Vec<String> = self
        .unfinished
        .iter()
        .filter_map(|n| n.borrow().tag().map(|t| t.to_string()))
        .collect();

      if open_tags.is_empty() && tag != Some("html") {
        self.add_tag("html".to_string());
      } else if open_tags == vec!["html"]
        && !matches!(tag, Some("head") | Some("body") | Some("/html"))
      {
        if tag.map(|t| HEAD_TAGS.contains(&t)).unwrap_or(false) {
          self.add_tag("head".to_string());
        } else {
          self.add_tag("body".to_string());
        }
      } else if open_tags == vec!["html", "head"]
        && !matches!(tag, Some("/head"))
        && !tag.map(|t| HEAD_TAGS.contains(&t)).unwrap_or(false)
      {
        self.add_tag("/head".to_string());
      } else {
        break;
      }
    }
  }
}

// pub fn print_tree(node: &Rc<RefCell<Node>>, indent: usize) {
//   let padding = " ".repeat(indent);
//   let borrowed = node.borrow();

//   match &*borrowed {
//     Node::Text(t) => println!("{}{:?}", padding, t.text),
//     Node::Element(e) => {
//       let mut s = String::new();
//       s.push_str(&format!("{}<{}", padding, e.tag));

//       for (key, value) in &e.attributes {
//         s.push_str(&format!(" {}=\"{}\"", key, value));
//       }

//       s.push('>');

//       println!("{}", s);
//       for child in &e.children {
//         print_tree(child, indent + 2);
//       }
//       println!("{}</{}>", padding, e.tag);
//       return;
//     }
//   }

//   for child in borrowed.children() {
//     print_tree(child, indent + 2);
//   }
// }
