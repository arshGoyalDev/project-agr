use crate::selector::{DescendantSelector, Selector, TagSelector};

use std::collections::HashMap;

pub fn inherited_properties() -> HashMap<&'static str, &'static str> {
  let mut map = HashMap::new();
  map.insert("font-size", "16px");
  map.insert("font-style", "normal");
  map.insert("font-weight", "normal");
  map.insert("font-family", "sans-serif");
  map.insert("color", "black");
  map
}

pub struct Rule {
  pub selector: Box<dyn Selector>,
  pub properties: HashMap<String, String>,
  pub priority: u32,
}

pub struct CSSParser {
  s: Vec<char>,
  i: usize,
}

impl CSSParser {
  pub fn new(s: &str) -> Self {
    CSSParser {
      s: s.chars().collect(),
      i: 0,
    }
  }

  fn whitespace(&mut self) {
    while self.i < self.s.len() && self.s[self.i].is_whitespace() {
      self.i += 1;
    }
  }

  fn word(&mut self) -> Result<String, String> {
    let start = self.i;
    while self.i < self.s.len() {
      let c = self.s[self.i];
      if c.is_alphanumeric() || "#-.%".contains(c) {
        self.i += 1;
      } else {
        break;
      }
    }
    if self.i == start {
      Err(format!(
        "Parsing Error: expected word at position {}",
        self.i
      ))
    } else {
      Ok(self.s[start..self.i].iter().collect())
    }
  }

  fn literal(&mut self, c: char) -> Result<(), String> {
    if self.i < self.s.len() && self.s[self.i] == c {
      self.i += 1;
      Ok(())
    } else {
      Err(format!(
        "Parsing Error: expected '{}' at position {}, got '{:?}'",
        c,
        self.i,
        self.s.get(self.i)
      ))
    }
  }

  fn ignore_until(&mut self, chars: &[char]) -> Option<char> {
    while self.i < self.s.len() {
      if chars.contains(&self.s[self.i]) {
        return Some(self.s[self.i]);
      }
      self.i += 1;
    }
    None
  }

  fn pair(&mut self) -> Result<(String, String), String> {
    let prop = self.word()?;
    self.whitespace();
    self.literal(':')?;
    self.whitespace();
    let val = self.word()?;
    Ok((prop.to_lowercase(), val))
  }

  pub fn body(&mut self) -> HashMap<String, String> {
    let mut pairs = HashMap::new();

    loop {
      self.whitespace();
      // Stop at end of string or closing brace
      if self.i >= self.s.len() || self.s[self.i] == '}' {
        break;
      }

      match self.pair() {
        Ok((prop, val)) => {
          pairs.insert(prop, val);
          self.whitespace();
          // consume the semicolon if present
          if self.i < self.s.len() && self.s[self.i] == ';' {
            self.i += 1;
          }
          self.whitespace();
        }
        Err(_) => {
          // Skip to next ';' or '}' and recover
          match self.ignore_until(&[';', '}']) {
            Some(';') => {
              self.i += 1; // consume ';'
              self.whitespace();
            }
            _ => break,
          }
        }
      }
    }

    pairs
  }

  fn selector(&mut self) -> Result<Box<dyn Selector>, String> {
    let tag = self.word()?;
    let mut out: Box<dyn Selector> = Box::new(TagSelector {
      tag: tag.to_lowercase(),
    });
    self.whitespace();

    while self.i < self.s.len() && self.s[self.i] != '{' {
      let tag = self.word()?;
      let descendant: Box<dyn Selector> = Box::new(TagSelector {
        tag: tag.to_lowercase(),
      });
      out = Box::new(DescendantSelector::new(out, descendant));
      self.whitespace();
    }

    Ok(out)
  }

  pub fn parse(&mut self) -> Vec<Rule> {
    let mut rules = Vec::new();

    loop {
      self.whitespace();
      if self.i >= self.s.len() {
        break;
      }

      match self.selector() {
        Ok(selector) => {
          match self.literal('{') {
            Ok(_) => {
              self.whitespace();
              let properties = self.body();
              let _ = self.literal('}'); // best-effort
              let priority = selector.priority();
              rules.push(Rule {
                selector,
                properties,
                priority,
              });
            }
            Err(_) => {
              // Skip malformed rule
              match self.ignore_until(&['}']) {
                Some(_) => {
                  self.i += 1;
                }
                None => break,
              }
            }
          }
        }
        Err(_) => match self.ignore_until(&['}']) {
          Some(_) => {
            self.i += 1;
          }
          None => break,
        },
      }
    }

    rules
  }
}
