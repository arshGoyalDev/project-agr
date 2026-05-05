use crate::selector::{ClassSelector, DescendantSelector, IdSelector, Selector, TagSelector};

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
      if c.is_alphanumeric() || "#-.%_".contains(c) {
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

  // NEW: Reads an entire CSS value until the semi-colon or closing brace
  // This allows for values with spaces, commas, quotes, etc.
  fn property_value(&mut self) -> Result<String, String> {
    let start = self.i;
    while self.i < self.s.len() {
      let c = self.s[self.i];
      if c == ';' || c == '}' {
        break;
      }
      self.i += 1;
    }
    if self.i == start {
      Err("Empty property value".to_string())
    } else {
      Ok(
        self.s[start..self.i]
          .iter()
          .collect::<String>()
          .trim()
          .to_string(),
      )
    }
  }

  fn pair(&mut self) -> Result<(String, String), String> {
    let prop = self.word()?;
    self.whitespace();
    self.literal(':')?;
    self.whitespace();

    // UPDATED: Use property_value instead of word to allow spaces/commas in values
    let val = self.property_value()?;
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
          if prop == "font" {
            for (p, v) in self.expand_font_shorthand(&val) {
              pairs.insert(p, v);
            }
          } else {
            pairs.insert(prop, val);
          }
          
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

  fn single_selector(&mut self) -> Result<Box<dyn Selector>, String> {
    let word = self.word()?;

    let mut out: Box<dyn Selector> = if word.starts_with('#') {
      Box::new(IdSelector {
        id: word[1..].to_lowercase(), // Strip the '#'
      })
    } else if word.starts_with('.') {
      Box::new(ClassSelector {
        class: word[1..].to_lowercase(), // Strip the '.'
      })
    } else {
      Box::new(TagSelector {
        tag: word.to_lowercase(),
      })
    };

    self.whitespace();

    // Stop if we hit `{` or `,`
    while self.i < self.s.len() && self.s[self.i] != '{' && self.s[self.i] != ',' {
      let word = self.word()?;

      let descendant: Box<dyn Selector> = if word.starts_with('#') {
        Box::new(IdSelector {
          id: word[1..].to_lowercase(),
        })
      } else if word.starts_with('.') {
        Box::new(ClassSelector {
          class: word[1..].to_lowercase(),
        })
      } else {
        Box::new(TagSelector {
          tag: word.to_lowercase(),
        })
      };

      out = Box::new(DescendantSelector::new(out, descendant));
      self.whitespace();
    }

    Ok(out)
  }

  fn selectors(&mut self) -> Result<Vec<Box<dyn Selector>>, String> {
    let mut sels = Vec::new();
    loop {
      self.whitespace();
      sels.push(self.single_selector()?);
      self.whitespace();

      if self.i < self.s.len() && self.s[self.i] == ',' {
        self.i += 1;
      } else {
        break;
      }
    }
    Ok(sels)
  }

  pub fn parse(&mut self) -> Vec<Rule> {
    let mut rules = Vec::new();

    loop {
      self.whitespace();
      if self.i >= self.s.len() {
        break;
      }

      match self.selectors() {
        Ok(selectors) => match self.literal('{') {
          Ok(_) => {
            self.whitespace();
            let properties = self.body();
            let _ = self.literal('}');

            for selector in selectors {
              let priority = selector.priority();
              rules.push(Rule {
                selector,
                properties: properties.clone(),
                priority,
              });
            }
          }
          Err(_) => match self.ignore_until(&['}']) {
            Some(_) => {
              self.i += 1;
            }
            None => break,
          },
        },
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
  
  fn expand_font_shorthand(&self, value: &str) -> HashMap<String, String> {
    let mut expanded = HashMap::new();
    let parts: Vec<&str> = value.split_whitespace().collect();
    
    for part in parts {
      match part.to_lowercase().as_str() {
        "italic" | "oblique" => {
          expanded.insert("font-style".to_string(), part.to_string());
        }
        "bold" => {
          expanded.insert("font-weight".to_string(), part.to_string());
        }
        p if p.contains('%') || p.contains("px") => {
          expanded.insert("font-size".to_string(), part.to_string());
        }
        _ => {
          expanded.insert("font-family".to_string(), part.to_string());
        }
      }
    }
    
    expanded
  }
}
