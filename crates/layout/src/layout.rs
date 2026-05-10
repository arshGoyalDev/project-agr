
pub const HSTEP: f32 = 9.0;
pub const VSTEP: f32 = 15.0;
pub const PRE_BG: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

pub const BLOCK_ELEMENTS: &[&str] = &[
  "html",
  "body",
  "article",
  "section",
  "nav",
  "aside",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "hgroup",
  "header",
  "footer",
  "address",
  "p",
  "hr",
  "pre",
  "blockquote",
  "ol",
  "ul",
  "menu",
  "li",
  "dl",
  "dt",
  "dd",
  "figure",
  "figcaption",
  "main",
  "div",
  "table",
  "form",
  "fieldset",
  "legend",
  "details",
  "summary",
  "canvas",
  "video",
  "audio",
  "noscript",
  "template",
  "caption",
  "thead",
  "tbody",
  "tfoot",
  "tr",
  "colgroup",
  "col",
];

pub fn decode_entities(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();

  while let Some(c) = chars.next() {
    if c != '&' {
      result.push(c);
      continue;
    }

    let mut entity = String::new();
    let mut terminated = false;

    for nc in chars.by_ref() {
      if nc == ';' {
        terminated = true;
        break;
      } else if nc.is_whitespace() {
        entity.push(nc);
        break;
      } else {
        entity.push(nc);
      }
    }

    if terminated {
      let replacement = match entity.as_str() {
        "lt" => Some("<"),
        "gt" => Some(">"),
        "amp" => Some("&"),
        "quot" => Some("\""),
        "apos" => Some("'"),
        "copy" => Some("©"),
        _ => None,
      };

      if let Some(r) = replacement {
        result.push_str(r);
      } else if entity.starts_with('#') {
        let code = if entity.starts_with("#x") || entity.starts_with("#X") {
          u32::from_str_radix(&entity[2..], 16).ok()
        } else {
          entity[1..].parse::<u32>().ok()
        };
        if let Some(n) = code {
          if let Some(ch) = char::from_u32(n) {
            result.push(ch);
          } else {
            result.push('&');
            result.push_str(&entity);
            result.push(';');
          }
        } else {
          result.push('&');
          result.push_str(&entity);
          result.push(';');
        }
      } else {
        result.push('&');
        result.push_str(&entity);
        result.push(';');
      }
    } else {
      result.push('&');
      result.push_str(&entity);
    }
  }

  result
}
