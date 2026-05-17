use iced::Task;

use crate::browser::Browser;
use crate::dom::{extract_title, find_inline_styles, find_stylesheet_links};
use crate::message::Message;
use crate::net::fetch_css_task;

use css_parser::CSSParser;
use css_parser::style;
use html_parser::HTMLParser;
use layout::{DocumentLayout, syntax_highlight};

pub fn html_fetched(
  browser: &mut Browser,
  tab_index: usize,
  base_url: String,
  is_view_source: bool,
  result: Result<String, String>,
) -> Task<Message> {
  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    if let Ok(body) = result {
      let mut html_parser = HTMLParser::new(body);
      let mut tree = html_parser.parse();

      if is_view_source {
        let highlighted = syntax_highlight(&tree);
        tree = HTMLParser::new(highlighted).parse();
      }

      let mut links = Vec::new();
      find_stylesheet_links(&tree, &mut links);

      tab.tree = Some(tree);

      if links.is_empty() {
        return Task::done(Message::CssFetched(tab_index, vec![]));
      } else {
        return Task::perform(fetch_css_task(links, base_url), move |bodies| {
          Message::CssFetched(tab_index, bodies)
        });
      }
    } else {
      tab.title = String::from("Network Error");
    }
  }

  Task::none()
}

pub fn css_fetched(
  browser: &mut Browser,
  tab_index: usize,
  css_bodies: Vec<String>,
) -> Task<Message> {
  let width = browser.width;

  if let Some(tab) = browser.tabs.get_mut(tab_index) {
    if let Some(tree) = &tab.tree {
      let default_css = include_str!("../../../../browser.css").to_string();
      let mut css_parser = CSSParser::new(&default_css);
      let mut rules = css_parser.parse();

      for body in css_bodies {
        let mut linked_parser = CSSParser::new(&body);
        rules.extend(linked_parser.parse());
      }

      let mut inline_style_texts = Vec::new();
      find_inline_styles(tree, &mut inline_style_texts);
      for css_text in inline_style_texts {
        rules.extend(CSSParser::new(&css_text).parse());
      }

      rules.sort_by_key(|r| r.priority);
      style(tree, &rules);

      tab.title = extract_title(tree).unwrap_or_else(|| tab.url.clone());

      let mut doc = DocumentLayout::new(tree);
      doc.layout(width);

      tab.display_list = doc.paint();
      tab.max_y = tab.display_list.max_y();
      tab.document = Some(doc);

      if let Some(fragment) = tab.url.split('#').nth(1) {
        if let Some(y_pos) = tab.document.as_ref().unwrap().get_element_y(fragment) {
          let max_scroll = (tab.max_y - width).max(0.0);
          tab.scroll_offset = y_pos.clamp(0.0, max_scroll);
        } else {
          tab.scroll_offset = 0.0;
        }
      } else {
        tab.scroll_offset = 0.0;
      }
    }
  }

  Task::none()
}
