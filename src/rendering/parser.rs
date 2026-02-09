use std::collections::HashMap;

pub fn parser(body: String, view_source: bool) -> String {
  if view_source {
    return body;
  }

  let mut text = String::new();
  let mut in_tag = false;
  let mut in_entity = false;
  let mut entity_value = String::new();

  let entities = {
    let mut map = HashMap::new();
    map.insert("gt", ">");
    map.insert("lt", "<");
    map.insert("amp", "&");
    map.insert("quot", "\"");
    map.insert("apos", "'");
    map
  };

  for c in body.chars() {
    if c == '<' {
      in_tag = true;
    } else if c == '>' {
      in_tag = false;
    } else if c == '&' {
      in_entity = true;
    } else if c == ';' && in_entity {
      in_entity = false;
      if let Some(entity) = entities.get(entity_value.as_str()) {
        text.push_str(entity);
      } else {
        text.push('&');
        text.push_str(&entity_value);
        text.push(';');
      }
      entity_value.clear();
    } else if in_entity {
      entity_value.push(c);
    } else if !in_tag {
      text.push(c);
    }
  }

  text
}