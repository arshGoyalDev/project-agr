use crate::node::{Element, Node, Text};

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

pub enum ParseYield {
  Finished(Rc<RefCell<Node>>),
  InlineScript { code: String },
  ExternalScript { src: String },
}

pub struct HTMLParser {
  chars: Vec<char>,
  pub pos: usize,
  unfinished: Vec<Rc<RefCell<Node>>>,
  head_closed: bool,
  pub deferred_scripts: Vec<String>,
  text: String,
  in_tag: bool,
  in_comment: bool,
  in_script: bool,
}

impl HTMLParser {
  pub fn new(body: String) -> Self {
    HTMLParser {
      chars: body.chars().collect(),
      pos: 0,
      unfinished: vec![],
      head_closed: false,
      deferred_scripts: vec![],
      text: String::new(),
      in_tag: false,
      in_comment: false,
      in_script: false,
    }
  }

  pub fn resume(&mut self) -> ParseYield {
    while self.pos < self.chars.len() {
      if self.in_comment {
        if self.chars[self.pos] == '-'
          && self.chars.get(self.pos + 1) == Some(&'-')
          && self.chars.get(self.pos + 2) == Some(&'>')
        {
          self.in_comment = false;
          self.pos += 3;
        } else {
          self.pos += 1;
        }
      } else if self.in_script {
        let close_tag = "</script>";
        let remaining_len = self.chars.len() - self.pos;

        if remaining_len >= close_tag.len() {
          let slice: String = self.chars[self.pos..self.pos + close_tag.len()]
            .iter()
            .collect();

          if slice.to_lowercase() == close_tag {
            if !self.text.is_empty() {
              self.add_text(self.text.clone());
              self.text.clear();
            }

            let mut is_async = false;
            let mut is_defer = false;
            let mut src = None;
            let mut code = String::new();

            {
              let script_node = self.unfinished.last().unwrap().clone();
              let borrow = script_node.borrow();
              if let Node::Element(e) = &*borrow {
                is_async = e.attributes.contains_key("async");
                is_defer = e.attributes.contains_key("defer");
                src = e.attributes.get("src").cloned();

                for child_rc in &e.children {
                  if let Node::Text(t) = &*child_rc.borrow() {
                    code.push_str(&t.text);
                  }
                }
              }
            }

            self.add_tag("/script".to_string());
            self.in_script = false;
            self.pos += close_tag.len();

            if let Some(url) = src {
              if !is_async && !is_defer {
                return ParseYield::ExternalScript { src: url };
              } else if is_defer {
                self.deferred_scripts.push(url);
              }
            } else {
              return ParseYield::InlineScript { code };
            }
            continue;
          }
        }

        self.text.push(self.chars[self.pos]);
        self.pos += 1;
      } else if !self.in_tag && self.chars[self.pos] == '<' {
        if self.chars.get(self.pos + 1) == Some(&'!')
          && self.chars.get(self.pos + 2) == Some(&'-')
          && self.chars.get(self.pos + 3) == Some(&'-')
        {
          if !self.text.is_empty() {
            self.add_text(self.text.clone());
            self.text.clear();
          }
          self.in_comment = true;
          self.pos += 4;
        } else {
          self.in_tag = true;
          if !self.text.is_empty() {
            self.add_text(self.text.clone());
          }
          self.text.clear();
          self.pos += 1;
        }
      } else if self.in_tag && self.chars[self.pos] == '>' {
        self.in_tag = false;
        let tag_content = self.text.clone();
        self.text.clear();

        self.add_tag(tag_content.clone());

        let trimmed = tag_content.trim().to_lowercase();
        if trimmed == "script" || trimmed.starts_with("script ") {
          self.in_script = true;
        }

        self.pos += 1;
      } else {
        self.text.push(self.chars[self.pos]);
        self.pos += 1;
      }
    }

    if !self.in_tag && !self.text.is_empty() {
      let t = self.text.clone();
      self.add_text(t);
      self.text.clear();
    }

    ParseYield::Finished(self.finish())
  }

  pub fn document(&self) -> Option<Rc<RefCell<Node>>> {
    self.unfinished.first().cloned()
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
      style: HashMap::new(),
    })));

    parent_rc.borrow_mut().children_mut().push(node);
  }

  fn add_tag(&mut self, tag: String) {
    let (tag, attributes) = self.get_attributes(&tag);

    if tag.starts_with('!') {
      return;
    }

    if tag == "/head" {
      self.head_closed = true;

      let body_open = self
        .unfinished
        .iter()
        .any(|n| n.borrow().tag().map(|t| t == "body").unwrap_or(false));
      if body_open {
        return;
      }
    }

    self.implicit_tags(Some(&tag.clone()));

    if tag.starts_with('/') {
      if self.unfinished.len() == 1 {
        return;
      }

      self.unfinished.pop().unwrap();
      // let parent_rc = self.unfinished.last().unwrap().clone();
      // parent_rc.borrow_mut().children_mut().push(node);
    } else if SELF_CLOSING_TAGS.contains(&tag.as_str()) {
      let parent_rc = self.unfinished.last().unwrap().clone();
      let parent_weak = Rc::downgrade(&parent_rc);

      let node = Rc::new(RefCell::new(Node::Element(Element {
        tag,
        self_closing: true,
        attributes,
        parent: Some(parent_weak),
        children: vec![],
        style: HashMap::new(),
      })));

      parent_rc.borrow_mut().children_mut().push(node);
    } else {
      let parent_weak = self.unfinished.last().map(|p| Rc::downgrade(p));

      let node = Rc::new(RefCell::new(Node::Element(Element {
        tag,
        self_closing: false,
        attributes,
        parent: parent_weak,
        children: vec![],
        style: HashMap::new(),
      })));

      if let Some(parent) = self.unfinished.last() {
        parent.borrow_mut().children_mut().push(node.clone());
      }

      self.unfinished.push(node);
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

        if value.len() >= 2
          && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
        {
          value = value[1..value.len() - 1].to_string();
        } else if value.starts_with('"') || value.starts_with('\'') {
          value = value.chars().skip(1).collect();
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
      self.unfinished.pop().unwrap();
      // let parent_rc = self.unfinished.last().unwrap().clone();
      // parent_rc.borrow_mut().children_mut().push(node);
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
        if tag.map(|t| HEAD_TAGS.contains(&t)).unwrap_or(false) && !self.head_closed {
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

pub fn print_tree(node: &Rc<RefCell<Node>>, indent: usize) {
  let padding = " ".repeat(indent);
  let borrowed = node.borrow();

  match &*borrowed {
    Node::Text(t) => println!("{}{:?}", padding, t.text),
    Node::Element(e) => {
      let mut s = String::new();
      s.push_str(&format!("{}<{}", padding, e.tag));

      for (key, value) in &e.attributes {
        s.push_str(&format!(" {}=\"{}\"", key, value));
      }

      s.push('>');
      println!("{}", s);

      for child in &e.children {
        print_tree(child, indent + 2);
      }

      println!("{}</{}>", padding, e.tag);
      return;
    }
  }

  for child in borrowed.children() {
    print_tree(child, indent + 2);
  }
}
